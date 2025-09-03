use crate::connector::make_text_payload;
use crate::synthesizer::config::Config;
use crate::synthesizer::StreamingRequest;
use crate::synthesizer::{Language, Voice};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_websockets::Message;

/// Creates a speech configuration message.
pub(crate) fn create_speech_config_message(request_id: String, config: &Config) -> Message {
    Message::text(make_text_payload(
        vec![
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "speech.config".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
        ],
        Some(
            &json!({"context":{"system":&config.device.system,"os":&config.device.os}}).to_string(),
        ),
    ))
}

/// Creates a speech context message.
pub(crate) fn create_synthesis_context_message(
    request_id: String,
    config: &Config,
    request: Option<&StreamingRequest>,
) -> Message {
    // derive voice name and language locale
    let language: Language = config.language.clone();
    let voice: Voice = config.voice.clone().unwrap_or(language.default_voice());

    let mut synthesis = json!({
        "audio": {
            "metadataOptions": {
                "bookmarkEnabled": config.bookmark_enabled,
                "punctuationBoundaryEnabled": config.punctuation_boundary_enabled,
                "sentenceBoundaryEnabled": config.sentence_boundary_enabled,
                "sessionEndEnabled": config.session_end_enabled,
                "visemeEnabled": config.viseme_enabled,
                "wordBoundaryEnabled": config.word_boundary_enabled
            },
            "outputFormat": config.audio_format.as_str()
        },
        "language": {
            "autoDetection": config.auto_detect_language
        },
        "voice": {
            "name": voice.as_str()
        }
    });

    // If auto-detection is disabled, include explicit locale
    if !config.auto_detect_language {
        synthesis["language"]["locale"] = json!(language.as_str());
    }

    // Apply optional per-request properties similar to Python's SpeechSynthesisRequest
    if let Some(req) = request {
        let mut req_json = json!({});
        if let Some(ref v) = req.pitch {
            req_json["pitch"] = json!(v);
        }
        if let Some(ref v) = req.rate {
            req_json["rate"] = json!(v);
        }
        if let Some(ref v) = req.volume {
            req_json["volume"] = json!(v);
        }
        if let Some(ref v) = req.style {
            req_json["style"] = json!(v);
        }
        if let Some(v) = req.temperature {
            req_json["temperature"] = json!(v);
        }
        if let Some(ref v) = req.prefer_locales {
            req_json["preferLocales"] = json!(v);
        }
        if let Some(ref v) = req.custom_lexicon_url {
            req_json["customLexiconUrl"] = json!(v);
        }
        synthesis["request"] = req_json;
    }

    Message::text(make_text_payload(
        vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "synthesis.context".to_string()),
        ],
        Some(&json!({ "synthesis": synthesis }).to_string()),
    ))
}

pub(crate) fn create_ssml_message(request_id: String, ssml: &str) -> Message {
    Message::text(make_text_payload(
        vec![
            (
                "Content-Type".to_string(),
                "application/ssml+xml".to_string(),
            ),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "ssml".to_string()),
        ],
        Some(ssml),
    ))
}

/// Creates a text streaming message (v2 endpoint only).
pub(crate) fn create_text_message(request_id: String, text: &str) -> Message {
    Message::text(make_text_payload(
        vec![
            (
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            ),
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "text".to_string()),
        ],
        Some(text),
    ))
}

/// Creates a turn.end message to finish the input text stream (v2 endpoint only).
pub(crate) fn create_turn_end_message(request_id: String) -> Message {
    Message::text(make_text_payload(
        vec![
            (
                "X-Timestamp".to_string(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            ),
            ("X-RequestId".to_string(), request_id),
            ("Path".to_string(), "turn.end".to_string()),
        ],
        None::<&str>,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{synthesizer::Config, Data, Message as EzMessage};

    #[test]
    fn test_create_speech_config_message() {
        let config = Config::new();
        let ws_msg = create_speech_config_message("id".to_string(), &config);
        let msg = EzMessage::try_from(ws_msg).unwrap();

        assert_eq!(msg.path, "speech.config");
        assert_eq!(msg.id, "id");
        assert_eq!(msg.get_header("Content-Type").unwrap(), "application/json");
        assert!(msg.get_header("X-Timestamp").is_some());

        match msg.data {
            Data::Text(Some(ref body)) => {
                let v: serde_json::Value = serde_json::from_str(body).unwrap();
                assert!(v.get("context").is_some());
            }
            _ => panic!("expected text body"),
        }
    }

    #[test]
    fn test_create_ssml_message() {
        let ws_msg = create_ssml_message("id".to_string(), "<speak>Hello</speak>");
        let msg = EzMessage::try_from(ws_msg).unwrap();

        assert_eq!(msg.path, "ssml");
        assert_eq!(msg.id, "id");
        assert_eq!(
            msg.get_header("Content-Type").unwrap(),
            "application/ssml+xml"
        );
        assert!(matches!(msg.data, Data::Text(Some(_))));
    }

    #[test]
    fn test_create_text_message() {
        let ws_msg = create_text_message("id".to_string(), "hello");
        let msg = EzMessage::try_from(ws_msg).unwrap();
        assert_eq!(msg.path, "text");
        assert_eq!(msg.id, "id");
        assert_eq!(
            msg.get_header("Content-Type").unwrap(),
            "text/plain; charset=utf-8"
        );
        match msg.data {
            Data::Text(Some(ref body)) => assert_eq!(body, "hello"),
            _ => panic!("expected text body"),
        }
    }

    #[test]
    fn test_create_turn_end_message() {
        let ws_msg = create_turn_end_message("id".to_string());
        let msg = EzMessage::try_from(ws_msg).unwrap();
        assert_eq!(msg.path, "turn.end");
        assert_eq!(msg.id, "id");
        assert!(msg.get_header("X-Timestamp").is_some());
        match msg.data {
            Data::Text(None) => {}
            _ => panic!("expected empty body"),
        }
    }
}
