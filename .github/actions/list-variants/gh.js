const core = require('@actions/core');
const github = require("@actions/github");

const ghContext = github.context;
const ghRepo = ghContext.payload.repository;
const repoOwner = ghRepo.owner;

const gh = github.getOctokit(core.getInput('token'));
const args = { owner: repoOwner.login, repo: ghRepo.name };

const files = new Set();

// Retrieve set of commits that are part of this pull request.
async function getCommits() {
    const url = ghContext.payload.pull_request.commits_url;
    let commits = await gh.paginate(`GET ${url}`, args);
	return commits;
}

// Get the details for a specific commit.
async function getCommit(commit) {
    args.ref = commit.id || commit.sha;
    return gh.rest.repos.getCommit(args);
}

// Inspect commit details to build list of modified file paths.
async function processCommitData(commit) {
    if (!commit || !commit.data) {
        return;
    }

    commit.data.files.forEach(file => {
        files.add(file.filename);
    });
}

// Build list of modified file paths included in this pull request.
function getChangedFiles(process) {
    getCommits().then(commits => {
        Promise.all(commits.map(getCommit))
            .then(data => Promise.all(data.map(processCommitData)))
            .then(() => {
                process(files);
            })
            .catch(console.log);
    });
}

// Define exports to be publicly accessible.
module.exports.getChangedFiles = getChangedFiles;

