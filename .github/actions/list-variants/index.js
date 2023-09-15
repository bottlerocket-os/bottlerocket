const core = require('@actions/core');
const github = require('./gh');
const repo = require('./repo');

// getInput returns a string, so check the boolean representation
const filterVariants = core.getInput("filter-variants") == "true";

// Determine if any changes impact a variant based on the paths that make up its tree
function isChanged(variantPaths, changedFiles) {
    for (var changedFile of changedFiles) {
        for (var variantPath of variantPaths) {
            if (changedFile.indexOf(variantPath) != -1) {
                core.debug("\tChanged file:", changedFile);
                return true;
            }
        }
    }
    return false;
}

// Sets a list of variants that feeds into subsequent matrix steps and formats a
// list of the variants that we want to skip for arm64.
function processVariants(changedFiles) {
    core.debug("processVariants called: " + changedFiles);
    let sourceCrates = null;
    if (filterVariants) { 
        sourceCrates = repo.getSourcePaths();
    }

    let { variantSets, aarchEnemies } = repo.discoverVariants(sourceCrates, filterVariants);

    // Check if we need to see what variants are affected by the changed files
    if (filterVariants) { 
        for (var variant in variantSets) {
            core.debug("Checking variant for changes:" + variant);
            if (!isChanged(variantSets[variant], changedFiles)) {
                delete variantSets[variant];
            }
        }
    }

    // Make sure we always return at least one variant
    let variants = Object.keys(variantSets);
    if (variants.length == 0) {
        variants.push("aws-dev");
    }

    // Print out the output for easy debugging and set the action variables to
    // be picked up in later steps.
    core.info("variants=" + JSON.stringify(variants));
    core.info("aarch-enemies=%" + JSON.stringify(aarchEnemies));
    core.setOutput('variants', variants);
    core.setOutput('aarch-enemies', aarchEnemies);
}

// Kick off the processing to get list of variants
if (filterVariants) {
    // Filter our set of variants based on the files being modified
    github.getChangedFiles(processVariants);
} else {
    // Just get the full set of variants
    processVariants(new Set());
}

