use super::types::LlmError;

pub struct TokenBudget {
    limit: u32,
    used: u32,
    warning_sent: bool,
}

impl TokenBudget {
    pub fn new(limit: u32) -> Self {
        Self { limit, used: 0, warning_sent: false }
    }

    pub fn record(&mut self, prompt_tokens: u32, completion_tokens: u32) -> Result<Option<u8>, LlmError> {
        self.used = self.used.saturating_add(prompt_tokens + completion_tokens);
        if self.used >= self.limit {
            return Err(LlmError::BudgetExhausted);
        }
        let pct = self.percent_used();
        if pct >= 80 && !self.warning_sent {
            self.warning_sent = true;
            return Ok(Some(pct));
        }
        Ok(None)
    }

    pub fn percent_used(&self) -> u8 {
        if self.limit == 0 { return 100; }
        ((self.used as u64 * 100) / self.limit as u64).min(100) as u8
    }

    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }

    pub fn check(&self) -> Result<(), LlmError> {
        if self.used >= self.limit {
            Err(LlmError::BudgetExhausted)
        } else {
            Ok(())
        }
    }
}
