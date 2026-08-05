/* [commitlint](https://github.com/conventional-changelog/commitlint) configuration
 *
 * Bottlerocket repos use a `scope: description` convention where the scope
 * is typically a package/component name (e.g. `twoliter: bump core-kit to v14.9.0`)
 * or an area (e.g. `changelog: update changelog for v1.64.0`).
 *
 * This differs from standard Conventional Commits which require a fixed type
 * prefix (feat, fix, etc.). We enforce the structural rules (colon separator,
 * line lengths, casing) while allowing any lowercase scope before the colon.
 */
import { RuleConfigSeverity } from "@commitlint/types";

// Custom plugin to validate the "scope: description" format used in Bottlerocket.
const kitScopePlugin = {
  rules: {
    // Validates that the header matches `lowercase-scope: description`
    "kit-scope-format": (parsed, _when, _value) => {
      const header = parsed.header;
      // Match: one or more lowercase words/numbers/dots/dashes, colon, space, then description
      const pattern = /^[a-z][a-z0-9._-]*: .+$/;
      return [
        pattern.test(header),
        "header must match the format 'scope: description' (e.g. 'twoliter: bump core-kit to v14.9.0')",
      ];
    },
  },
};

export default {
  plugins: [kitScopePlugin],
  rules: {
    // Structural rules
    "header-max-length": [RuleConfigSeverity.Error, "always", 72],
    "header-trim": [RuleConfigSeverity.Error, "always"],
    "body-max-line-length": [RuleConfigSeverity.Error, "always", 72],
    "body-leading-blank": [RuleConfigSeverity.Error, "always"],

    // Subject rules (applied to text after the colon)
    "subject-full-stop": [RuleConfigSeverity.Error, "never", "."],

    // Custom kit scope format
    "kit-scope-format": [RuleConfigSeverity.Error, "always"],
  },
  ignores: [
    (message) => message.includes("Merge pull request #"),
    (message) => message.startsWith("Revert \""),
  ],
};
