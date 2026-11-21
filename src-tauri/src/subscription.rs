use crate::headless;
use crate::models::{ExecutionSource, PluginEvent, SubscriptionConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SubscriptionEngine {
    subscriptions: Arc<RwLock<Vec<SubscriptionConfig>>>,
}

impl SubscriptionEngine {
    pub fn new(subscriptions: Vec<SubscriptionConfig>) -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(subscriptions)),
        }
    }

    pub async fn reload(&self, subscriptions: Vec<SubscriptionConfig>) {
        let mut subs = self.subscriptions.write().await;
        *subs = subscriptions;
    }

    pub async fn process_event(
        &self,
        plugin_id: &str,
        plugin_name: &str,
        event: &PluginEvent,
        app_handle: &tauri::AppHandle,
    ) {
        let subs = self.subscriptions.read().await;
        for sub in subs.iter() {
            if !sub.enabled {
                continue;
            }

            let name_match = sub.plugin_name == "*"
                || sub.plugin_name == plugin_id
                || sub.plugin_name == plugin_name;
            let type_match = sub.event_type == "*" || sub.event_type == event.event_type;

            if !name_match || !type_match {
                continue;
            }

            let prompt = expand_template(&sub.prompt_template, &event.data);
            let wd = sub.working_dir.clone();
            let args = sub.claude_args.clone();
            let source = ExecutionSource::Plugin {
                plugin_name: plugin_name.to_string(),
                event_type: event.event_type.clone(),
            };
            let app = app_handle.clone();

            tokio::spawn(async move {
                if let Err(e) = headless::execute(&prompt, wd.as_deref(), &args, source, &app).await
                {
                    eprintln!("Subscription execution failed: {}", e);
                }
            });
        }
    }
}

fn expand_template(template: &str, data: &serde_json::Value) -> String {
    let mut result = template.to_string();
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", key);
            let raw = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let replacement = format!("<data key=\"{}\">{}</data>", key, raw);
            result = result.replace(&placeholder, &replacement);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_template_basic() {
        let data = serde_json::json!({"name": "test"});
        let result = expand_template("Hello {{name}}!", &data);
        assert_eq!(result, r#"Hello <data key="name">test</data>!"#);
    }

    #[test]
    fn expand_template_multiple_placeholders() {
        let data = serde_json::json!({"file": "main.rs", "line": "42"});
        let result = expand_template("Error in {{file}} at line {{line}}", &data);
        assert_eq!(
            result,
            r#"Error in <data key="file">main.rs</data> at line <data key="line">42</data>"#
        );
    }

    #[test]
    fn expand_template_non_string_values() {
        let data = serde_json::json!({"count": 5, "active": true});
        let result = expand_template("Items: {{count}}, Active: {{active}}", &data);
        assert_eq!(
            result,
            r#"Items: <data key="count">5</data>, Active: <data key="active">true</data>"#
        );
    }

    #[test]
    fn expand_template_missing_key_kept() {
        let data = serde_json::json!({"name": "test"});
        let result = expand_template("{{name}} {{missing}}", &data);
        assert_eq!(result, r#"<data key="name">test</data> {{missing}}"#);
    }

    #[test]
    fn expand_template_no_object_data() {
        let data = serde_json::json!("not an object");
        let result = expand_template("Hello {{name}}", &data);
        assert_eq!(result, "Hello {{name}}");
    }

    #[test]
    fn expand_template_empty_template() {
        let data = serde_json::json!({"name": "test"});
        let result = expand_template("", &data);
        assert_eq!(result, "");
    }

    #[test]
    fn expand_template_empty_data() {
        let data = serde_json::json!({});
        let result = expand_template("Hello {{name}}!", &data);
        assert_eq!(result, "Hello {{name}}!");
    }

    #[test]
    fn expand_template_nested_object() {
        let data = serde_json::json!({"user": {"id": 1, "name": "test"}});
        let result = expand_template("User: {{user}}", &data);
        assert_eq!(
            result,
            r#"User: <data key="user">{"id":1,"name":"test"}</data>"#
        );
    }

    mod props {
        use super::super::*;
        use proptest::collection::{btree_map, vec};
        use proptest::prelude::*;
        use serde_json::Value;

        // Template text and keys exclude `{` / `}` so that the only `{{key}}`
        // occurrences in a generated template are the ones the test inserts.
        // Arbitrary braces could form placeholders by accident, which would
        // make the expected output depend on the input in ways unrelated to
        // the property being checked.
        fn text() -> impl Strategy<Value = String> {
            "[^{}]{0,12}"
        }

        fn key() -> impl Strategy<Value = String> {
            "[^{}]{0,8}"
        }

        fn placeholder(key: &str) -> String {
            format!("{{{{{key}}}}}")
        }

        fn single_entry(key: &str, value: String) -> Value {
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), Value::String(value));
            Value::Object(map)
        }

        fn non_object_json() -> impl Strategy<Value = Value> {
            prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::from),
                any::<i64>().prop_map(Value::from),
                any::<String>().prop_map(Value::from),
                vec(any::<String>(), 0..4).prop_map(Value::from),
            ]
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(64))]

            /// Every placeholder for a present key becomes exactly the `<data>` element;
            /// the value is inserted verbatim (any string, braces included).
            #[test]
            fn present_key_expands_exactly(
                segments in vec(text(), 1..5),
                key in key(),
                value in any::<String>(),
            ) {
                let template = segments.join(&placeholder(&key));
                let replacement = format!("<data key=\"{key}\">{value}</data>");
                let data = single_entry(&key, value);
                prop_assert_eq!(expand_template(&template, &data), segments.join(&replacement));
            }

            /// No `{{key}}` remains for a present key. The value is brace-free here because
            /// values are inserted without escaping, so a value that itself contains
            /// `{{key}}` would (by design) reappear in the output.
            #[test]
            fn present_key_leaves_no_placeholder(
                segments in vec(text(), 2..5),
                key in key(),
                value in text(),
            ) {
                let template = segments.join(&placeholder(&key));
                let data = single_entry(&key, value);
                let result = expand_template(&template, &data);
                prop_assert!(!result.contains(&placeholder(&key)));
            }

            #[test]
            fn missing_key_placeholder_is_kept(
                segments in vec(text(), 2..5),
                (present, missing) in (key(), key()).prop_filter("distinct keys", |(a, b)| a != b),
                value in any::<String>(),
            ) {
                let template = segments.join(&placeholder(&missing));
                let data = single_entry(&present, value);
                prop_assert_eq!(expand_template(&template, &data), template);
            }

            #[test]
            fn template_without_placeholder_is_unchanged(
                template in any::<String>().prop_filter("no `{{`", |t| !t.contains("{{")),
                data in btree_map(any::<String>(), any::<String>(), 0..4),
            ) {
                let data = serde_json::to_value(data).unwrap();
                prop_assert_eq!(expand_template(&template, &data), template);
            }

            #[test]
            fn non_object_data_leaves_template_unchanged(
                segments in vec(text(), 1..5),
                key in key(),
                data in non_object_json(),
            ) {
                let template = segments.join(&placeholder(&key));
                prop_assert_eq!(expand_template(&template, &data), template);
            }
        }
    }
}
