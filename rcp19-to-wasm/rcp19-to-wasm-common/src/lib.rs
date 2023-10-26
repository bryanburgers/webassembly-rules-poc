use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Rules {
    pub value: Vec<Rule>,
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "FieldName")]
    pub field_name: String,
    #[serde(rename = "RuleAction")]
    pub rule_action: RuleAction,
    #[serde(rename = "RuleMessage")]
    pub rule_message: String,
    #[serde(rename = "RuleExpression")]
    pub rule_expression: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleAction {
    Evaluate,
    Accept,
    Reject,
    Warning,
    Set,
    SetDisplay,
    SetRequired,
}
