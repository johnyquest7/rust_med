/// Constants for the medical note generation application
/// 
/// This module contains the prompt template and configuration constants
/// used for generating medical notes from transcripts. The prompt is designed
/// to be easily editable and iterated upon.

/// The prompt template for generating medical notes from transcripts
/// 
/// This prompt instructs the AI model to convert doctor-patient conversation
/// transcripts into structured SOAP format medical notes.
pub const MEDICAL_NOTE_PROMPT: &str = r#"Given the following doctor-patient conversation transcript, create a concise medical note in SOAP format:

TRANSCRIPT:
{transcript}

Please provide a well-structured medical note with:
- Subjective: Patient's chief complaint and symptoms
- Objective: Observable findings and vital signs if mentioned
- Assessment: Clinical impression or diagnosis
- Plan: Treatment plan and follow-up instructions

MEDICAL NOTE:"#;

/// Temperature setting for medical note generation
/// 
/// Lower values (0.1-0.3) provide more focused, consistent output
/// Higher values (0.5-0.8) provide more creative but less consistent output
pub const TEMPERATURE: &str = "0.3";
