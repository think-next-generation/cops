//! Question command handlers

//!
//! Structured Q&A system for task clarification.

use super::args::QuestionCommands;
use super::ctx::Ctx;
use super::output::{self, Format};
use crate::core::{CreateQuestion, AnswerQuestion, QuestionType, Urgency};
use crate::error::{Error, Result};
use uuid::Uuid;

pub async fn handle(cmd: QuestionCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        QuestionCommands::Create {
            task_id,
            question,
            qtype,
            option,
            urgency,
        } => handle_create(ctx, task_id, question, qtype, option, urgency).await,
        QuestionCommands::List {
            task,
            unanswered,
            urgent,
        } => handle_list(ctx, task, unanswered, urgent).await,
        QuestionCommands::Answer {
            id,
            answer,
            by,
        } => handle_answer(ctx, id, answer, by).await,
    }
}

async fn handle_create(
    ctx: &Ctx,
    task_id: String,
    question_text: String,
    qtype: String,
    options: Vec<String>,
    urgency: String,
) -> Result<()> {
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    // Verify task exists
    let task_repo = ctx.task_repo();
    task_repo.find_by_id(task_uuid).await?
        .ok_or_else(|| Error::Custom(format!("Task not found: {}", task_id)))?;

    // Parse question type
    let question_type: QuestionType = match qtype.to_lowercase().as_str() {
        "open" => QuestionType::OpenEnded,
        "single" => QuestionType::SingleChoice,
        "multi" => QuestionType::MultiChoice,
        _ => return Err(Error::Parse(format!("Invalid question type: {}", qtype))),
    };

    // Parse urgency
    let urgency: Urgency = urgency.parse()
        .map_err(|e: String| Error::Parse(e))?;

    // Validate options for choice types
    if matches!(question_type, QuestionType::SingleChoice | QuestionType::MultiChoice) {
        if options.len() < 2 {
            return Err(Error::Custom("Choice questions require at least 2 options".to_string()));
        }
    }

    let input = CreateQuestion {
        question_text,
        question_type,
        options: if options.is_empty() { None } else { Some(options) },
        urgency,
    };

    let repo = ctx.question_repo();
    let question = repo.create(task_uuid, &input).await?;

    println!("Created question: {}", question.id);
    println!("  Task: {}", task_id);
    println!("  Type: {:?}", question.question_type);
    println!("  Urgency: {:?}", question.urgency);

    Ok(())
}

async fn handle_list(
    ctx: &Ctx,
    task: Option<String>,
    unanswered: bool,
    urgent: bool,
) -> Result<()> {
    let task_id = task
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| Error::Parse(e.to_string()))?;

    let repo = ctx.question_repo();
    let mut questions = if let Some(tid) = task_id {
        repo.find_by_task(tid).await?
    } else {
        // Get all questions by finding all tasks first
        let task_repo = ctx.task_repo();
        let tasks = task_repo.find_all(&Default::default()).await?;
        let mut all_questions = Vec::new();
        for task in tasks {
            let task_questions = repo.find_by_task(task.id).await?;
            all_questions.extend(task_questions);
        }
        all_questions
    };

    // Filter by unanswered
    if unanswered {
        questions.retain(|q| !q.is_answered());
    }

    // Filter by urgent
    if urgent {
        questions.retain(|q| matches!(q.urgency, Urgency::High));
    }

    output::print_questions(&questions, Format::Table);

    Ok(())
}

async fn handle_answer(
    ctx: &Ctx,
    id: String,
    answer: String,
    by: Option<String>,
) -> Result<()> {
    let question_id = Uuid::parse_str(&id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    let input = AnswerQuestion {
        answer,
        answered_by: by,
    };

    let repo = ctx.question_repo();
    let question = repo.answer(question_id, &input).await?;

    let id_str = id.to_string();
    let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
    println!("Answered question: {}", id_short);
    println!("  Answer: {}", question.answer.unwrap_or_default());

    Ok(())
}
