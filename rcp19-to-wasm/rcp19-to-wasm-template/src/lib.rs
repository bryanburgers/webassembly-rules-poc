use rcp19_to_wasm_common::{RuleAction, Rules};
use std::collections::BTreeMap;

mod reso;

#[no_mangle]
pub extern "C" fn validate_target(ptr: *const u8, len: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let rules_data: &str = match std::str::from_utf8(bytes) {
        Ok(data) => data,
        Err(_) => {
            reso::diagnostic("Failed to load rules data string");
            return;
        }
    };
    let rules: Rules = match serde_json::from_str(rules_data) {
        Ok(rules) => rules,
        Err(err) => {
            reso::diagnostic(&format!("Failed to load rules data: {err}"));
            return;
        }
    };

    let data: serde_json::Value = reso::data();
    #[allow(unused)]
    let previous_data: Option<serde_json::Value> = reso::previous_data();

    let mut locals = BTreeMap::default();

    let mut engine = rets_expression::Engine::default();
    engine.set_function("NOW", Box::new(NowFunction));
    engine.set_function("TODAY", Box::new(TodayFunction));

    let mut expressions = Vec::new();
    for rule in &rules.value {
        match rule.rule_expression.parse::<rets_expression::Expression>() {
            Ok(expression) => expressions.push(expression),
            Err(err) => {
                reso::diagnostic(&format!("Unexpectedly failed to parse expression: {err}"));
                return;
            }
        }
    }

    for (idx, rule) in rules.value.iter().enumerate() {
        let expression = expressions.get(idx).unwrap();

        let context = rets_expression::EvaluateContext::new(&engine, &data);
        context.set_previous(previous_data.as_ref());

        let result = expression.apply_with_locals(context, &locals);

        match rule.rule_action {
            RuleAction::Evaluate => match result {
                Ok(value) => {
                    reso::diagnostic(&format!(
                        "{}: {}",
                        rule.rule_message,
                        serde_json::to_string(value.as_ref()).unwrap(),
                    ));
                }
                Err(err) => {
                    reso::diagnostic(&format!("{}: {:?}", rule.rule_message, err));
                }
            },
            RuleAction::Accept => match result {
                Ok(value) if value.as_bool() == Some(true) => {
                    // Accept if true
                }
                _ => {
                    // Reject if any other value or an error
                    reso::error(&rule.field_name, &rule.rule_message);
                }
            },
            RuleAction::Reject => match result {
                Ok(value) if value.as_bool() == Some(true) => {
                    // Reject if true
                    reso::error(&rule.field_name, &rule.rule_message);
                }
                _ => {
                    // Accept if any other value or an error
                }
            },
            RuleAction::Warning => match result {
                Ok(value) if value.as_bool() == Some(true) => {
                    // Warn if true
                    reso::warn(&rule.field_name, &rule.rule_message);
                }
                _ => {
                    // Accept if any other value or an error
                }
            },
            RuleAction::SetRequired => match result {
                Ok(value) if value.as_bool() == Some(true) => {
                    // Set required if true
                    reso::set_required(&rule.field_name, true);
                }
                _ => {
                    // Otherwise not required
                    reso::set_required(&rule.field_name, false);
                }
            },
            RuleAction::SetDisplay => match result {
                Ok(value) if value.as_bool() == Some(true) => {
                    // Set display if true
                    reso::set_display(&rule.field_name, true);
                }
                _ => {
                    // Otherwise don't display
                    reso::set_display(&rule.field_name, false);
                }
            },
            RuleAction::Set => match result {
                Ok(value) => {
                    reso::set(&rule.field_name, value.as_ref());
                    locals.insert(&rule.field_name, value);
                }
                Err(_) => {
                    // Do nothing.
                }
            },
        }
    }
}

struct TodayFunction;

impl rets_expression::function::Function<()> for TodayFunction {
    fn evaluate<'json>(
        &self,
        _context: rets_expression::function::FunctionContext<'_, ()>,
        _input: Vec<std::borrow::Cow<'json, serde_json::Value>>,
    ) -> Result<std::borrow::Cow<'json, serde_json::Value>, rets_expression::function::FunctionError>
    {
        // The proposed spec does not have a way to get the current time. This is an oversight meant
        // to simplify the proposal for understanding. Until that is fixed, return a hardcoded date.
        reso::diagnostic("TODAY() called. Using hardcoded 2023-04-21");
        Ok(std::borrow::Cow::Owned(serde_json::Value::String(
            String::from("2023-04-21"),
        )))
    }
}

struct NowFunction;

impl rets_expression::function::Function<()> for NowFunction {
    fn evaluate<'json>(
        &self,
        _context: rets_expression::function::FunctionContext<'_, ()>,
        _input: Vec<std::borrow::Cow<'json, serde_json::Value>>,
    ) -> Result<std::borrow::Cow<'json, serde_json::Value>, rets_expression::function::FunctionError>
    {
        // The proposed spec does not have a way to get the current time. This is an oversight meant
        // to simplify the proposal for understanding. Until that is fixed, return a hardcoded
        // timestamp.
        reso::diagnostic("NOW() called. Using hardcoded 2023-04-21T00:00:00.000Z");
        Ok(std::borrow::Cow::Owned(serde_json::Value::String(
            String::from("2023-04-21T00:00:00.000Z"),
        )))
    }
}
