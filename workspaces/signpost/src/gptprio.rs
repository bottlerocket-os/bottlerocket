use std::fmt;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GptPrio(u64);

impl GptPrio {
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn priority(self) -> u8 {
        (self.0 >> 48) as u8 & 0xf
    }

    /// Panics if `priority > 15`.
    pub(crate) fn set_priority(&mut self, priority: u8) {
        if priority > 0xf {
            panic!("priority cannot be greater than 15");
        }

        self.0 = (self.0 & !(0xf_u64 << 48)) | (u64::from(priority) << 48);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn tries_left(self) -> u8 {
        (self.0 >> 52) as u8 & 0xf
    }

    /// Panics if `tries_left > 15`.
    pub(crate) fn set_tries_left(&mut self, tries_left: u8) {
        if tries_left > 15 {
            panic!("tries_left cannot be greater than 15");
        }

        self.0 = (self.0 & !(0xf_u64 << 52)) | (u64::from(tries_left) << 52);
    }

    pub(crate) fn successful(self) -> bool {
        (self.0 >> 56) & 1 == 1
    }

    pub(crate) fn set_successful(&mut self, successful: bool) {
        self.0 = (self.0 & !(1_u64 << 56)) | (if successful { 1 } else { 0 } << 56);
    }

    pub(crate) fn will_boot(self) -> bool {
        self.priority() > 0 || self.successful()
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    }
}
