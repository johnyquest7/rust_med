// The r#".."# means string literal raw string
// This is used to avoid having to escape backslashes in the prompt

#[allow(dead_code)]
pub const MEDICAL_NOTE_PROMPT: &str = r#"Given the following doctor-patient conversation transcript, create a concise medical note in SOAP format:

TRANSCRIPT:
{transcript}

Please provide a well-structured medical note with:
- Subjective: Patient's chief complaint and symptoms
- Objective: Observable findings and vital signs if mentioned
- Assessment: Clinical impression or diagnosis
- Plan: Treatment plan and follow-up instructions

MEDICAL NOTE:"#;

#[allow(dead_code)]
pub const TEMPERATURE: &str = "0.3";
