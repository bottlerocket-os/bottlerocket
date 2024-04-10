use bloodhound::results::{CheckStatus, Checker, CheckerMetadata, CheckerResult, Mode};
use bloodhound::*;

const CRYPTO_FIPS_ENABLED: &str = "/proc/sys/crypto/fips_enabled";
const EXPECTED_FIPS_ENABLED: &str = "1";

const CRYPTO_FIPS_NAME: &str = "/proc/sys/crypto/fips_name";
const EXPECTED_FIPS_NAME: &str = "Amazon Linux 2023 Kernel Cryptographic API";

const FIPS_KERNEL_CHECK_MARKER: &str = "/etc/.fips-kernel-check-passed";
const FIPS_MODULE_CHECK_MARKER: &str = "/etc/.fips-module-check-passed";

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct FIPS01000000Checker {}

impl Checker for FIPS01000000Checker {
    fn execute(&self) -> CheckerResult {
        check_file_contains!(
            CRYPTO_FIPS_ENABLED,
            &[EXPECTED_FIPS_ENABLED],
            format!("{CRYPTO_FIPS_ENABLED} != {EXPECTED_FIPS_ENABLED}"),
            format!("{CRYPTO_FIPS_ENABLED} not found")
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "FIPS mode is enabled.".to_string(),
            id: "1.0".to_string(),
            level: 0,
            name: "fips01000000".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct FIPS01010000Checker {}

impl Checker for FIPS01010000Checker {
    fn execute(&self) -> CheckerResult {
        check_file_contains!(
            CRYPTO_FIPS_NAME,
            &[EXPECTED_FIPS_NAME],
            format!("{CRYPTO_FIPS_NAME} != '{EXPECTED_FIPS_NAME}'"),
            format!("{CRYPTO_FIPS_NAME} not found")
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: format!("FIPS module is {EXPECTED_FIPS_NAME}.").to_string(),
            id: "1.1".to_string(),
            level: 0,
            name: "fips01010000".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct FIPS01020000Checker {}

impl Checker for FIPS01020000Checker {
    fn execute(&self) -> CheckerResult {
        let result = check_file_exists!(
            FIPS_KERNEL_CHECK_MARKER,
            format!("{FIPS_KERNEL_CHECK_MARKER} not found")
        );

        // Check if we need to continue
        if result.status == CheckStatus::FAIL {
            return result;
        }

        check_file_exists!(
            FIPS_MODULE_CHECK_MARKER,
            format!("{FIPS_MODULE_CHECK_MARKER} not found")
        )
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "FIPS self-tests passed.".to_string(),
            id: "1.2".to_string(),
            level: 0,
            name: "fips01020000".to_string(),
            mode: Mode::Automatic,
        }
    }
}
