use bit_field::BitField;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GptPrio(u64);

impl GptPrio {
    pub(crate) fn priority(self) -> u64 {
        self.0.get_bits(48..52)
    }

    /// Panics if `priority > 15`.
    pub(crate) fn set_priority(&mut self, priority: u64) {
        self.0.set_bits(48..52, priority);
    }

    pub(crate) fn tries_left(self) -> u64 {
        self.0.get_bits(52..56)
    }

    /// Panics if `tries_left > 15`.
    pub(crate) fn set_tries_left(&mut self, tries_left: u64) {
        self.0.set_bits(52..56, tries_left);
    }

    pub(crate) fn successful(self) -> bool {
        self.0.get_bit(56)
    }

    pub(crate) fn set_successful(&mut self, successful: bool) {
        self.0.set_bit(56, successful);
    }

    pub(crate) fn will_boot(self) -> bool {
        (self.priority() > 0 && self.tries_left() > 0) || self.successful()
    }

    pub(crate) fn boot_has_succeeded(&mut self) {
        self.0.set_bit(57, true);
    }

    pub(crate) fn has_boot_succeeded(&self) -> bool {
        self.0.get_bit(57)
    }
}

impl From<u64> for GptPrio {
    fn from(flags: u64) -> Self {
        Self(flags)
    }
}

impl From<GptPrio> for u64 {
    fn from(flags: GptPrio) -> Self {
        flags.0
    }
}

impl fmt::Display for GptPrio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "priority={} tries_left={} successful={}",
            self.priority(),
            self.tries_left(),
            self.successful()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::gptprio::GptPrio;

    #[test]
    fn test() {
        let mut prio = GptPrio(0x5555555555555555);
        assert_eq!(prio.priority(), 5);
        assert_eq!(prio.tries_left(), 5);
        assert_eq!(prio.successful(), true);
        assert_eq!(prio.will_boot(), true);
        prio.set_priority(0);
        assert_eq!(prio.0, 0x5550555555555555);
        prio.set_tries_left(0);
        assert_eq!(prio.0, 0x5500555555555555);
        prio.set_successful(false);
        assert_eq!(prio.0, 0x5400555555555555);
        assert_eq!(prio.will_boot(), false);

        prio = GptPrio(0x0000000000000000);
        assert_eq!(prio.has_boot_succeeded(), false);
        prio.boot_has_succeeded();
        assert_eq!(prio.has_boot_succeeded(), true);
    }
}
