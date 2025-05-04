use ollama_rs::Ollama;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use crate::decoding::arithmetic_to_lambda;
use crate::expr::LambdaExpr;
use crate::reduction::beta_reduce_step;

const SYSTEM_PROMPT_0: &str = "You are an accurate AI model tasked with translating a user's query \
into a mathematical expression. You will ONLY output the expression, with NO parenthesis. The \
expression MUST represent the question asked by the user. DO NOT simplify OR evaluate it. Use + to
represent addition, - for subtraction, * for multiplication, / for division, and ! for factorial.
EXAMPLE
USER: What is three plus seven divided by twelve?
YOU: 3 + 7 / 12
";

pub(crate) fn instantiate_ollama() -> Ollama {
    Ollama::default()
}

pub(crate) async fn handle_prompt(prompt: String, ollama: &mut Ollama) -> Vec<LambdaExpr> {
    let mut history = vec![];
    let res = ollama
        .send_chat_messages_with_history(
            &mut history,
            ChatMessageRequest::new(
                "llama3:latest".to_string(),
                vec![ChatMessage::system(SYSTEM_PROMPT_0.to_string()), ChatMessage::user(prompt)]
            )
        )
        .await;
    let output = res.unwrap().message.content;
    let message = output.replace("!", " !");
    let mut expr = arithmetic_to_lambda(&message);
    let mut terms = vec![expr.clone()];
    while beta_reduce_step(&mut expr) { terms.push(expr.clone()) }
    terms
}