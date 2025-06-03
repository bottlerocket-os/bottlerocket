use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

// We added incorrect hostname patterns to be matched by the ECR credential provider.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "settings.kubernetes.credential-providers.ecr-credential-provider.image-patterns",
        old_vals: &[
            "*.dkr.ecr.*.amazonaws.com",
            "*.dkr.ecr.*.amazonaws.com.cn",
            "*.dkr.ecr.*.on.aws",
            "*.dkr.ecr.*.on.amazonwebservices.com.cn",
            "*.dkr.ecr-fips.*.amazonaws.com",
            "*.dkr.ecr.*.cloud.adc-e.uk",
            "*.dkr.ecr-fips.*.cloud.adc-e.uk",
            "*.dkr.ecr.*.c2s.ic.gov",
            "*.dkr.ecr-fips.*.c2s.ic.gov",
            "*.dkr.ecr.*.sc2s.sgov.gov",
            "*.dkr.ecr-fips.*.sc2s.sgov.gov",
            "*.dkr.ecr.*.csp.hci.ic.gov",
            "*.dkr.ecr-fips.*.csp.hci.ic.gov",
            "public.ecr.aws",
        ],
        new_vals: &[
            "*.dkr.ecr.*.amazonaws.com",
            "*.dkr.ecr.*.amazonaws.com.cn",
            "*.dkr-ecr.*.on.aws",
            "*.dkr-ecr.*.on.amazonwebservices.com.cn",
            "*.dkr.ecr-fips.*.amazonaws.com",
            "*.dkr.ecr.*.cloud.adc-e.uk",
            "*.dkr.ecr-fips.*.cloud.adc-e.uk",
            "*.dkr.ecr.*.c2s.ic.gov",
            "*.dkr.ecr-fips.*.c2s.ic.gov",
            "*.dkr.ecr.*.sc2s.sgov.gov",
            "*.dkr.ecr-fips.*.sc2s.sgov.gov",
            "*.dkr.ecr.*.csp.hci.ic.gov",
            "*.dkr.ecr-fips.*.csp.hci.ic.gov",
            "public.ecr.aws",
        ],
    }]))
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
