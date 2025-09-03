use crate::connector::Client as BaseClient;
use crate::synthesizer::utils::{create_text_message, create_turn_end_message};

const MAX_TEXT_FRAME_BYTES: usize = 4096;

#[derive(Clone)]
pub struct TextStream {
    client: BaseClient,
    request_id: String,
}

impl TextStream {
    pub(crate) fn new(client: BaseClient, request_id: String) -> Self {
        Self { client, request_id }
    }

    /// Write text to the streaming request. Large inputs are chunked to <= 4096 bytes.
    pub async fn write(&self, text: &str) -> crate::Result<()> {
        for chunk in Utf8Chunker::new(text.as_bytes(), MAX_TEXT_FRAME_BYTES) {
            let chunk_str = std::str::from_utf8(chunk)
                .map_err(|e| crate::Error::InternalError(e.to_string()))?;
            self.client
                .send(create_text_message(self.request_id.clone(), chunk_str))
                .await?;
        }
        Ok(())
    }

    /// Finish the input stream by sending a turn.end message.
    pub async fn finish(&self) -> crate::Result<()> {
        self.client
            .send(create_turn_end_message(self.request_id.clone()))
            .await
    }
}

/// Iterator that yields UTF-8 safe slices not exceeding `limit` bytes.
struct Utf8Chunker<'a> {
    data: &'a [u8],
    limit: usize,
    offset: usize,
}

impl<'a> Utf8Chunker<'a> {
    fn new(data: &'a [u8], limit: usize) -> Self {
        Self { data, limit, offset: 0 }
    }
}

impl<'a> Iterator for Utf8Chunker<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        let remaining = &self.data[self.offset..];
        let take = remaining.len().min(self.limit);
        let mut end = self.offset + take;

        // back off until end is on a UTF-8 boundary
        while end > self.offset && (self.data[end - 1] & 0b1100_0000) == 0b1000_0000 {
            end -= 1;
        }

        if end == self.offset {
            // extremely rare: individual codepoint longer than limit; fall back to single byte advance
            end = self.offset + take;
        }

        let chunk = &self.data[self.offset..end];
        self.offset = end;
        Some(chunk)
    }
}


