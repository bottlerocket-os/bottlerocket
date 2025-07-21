# Git Workflow for Bottlerocket Fork

This document describes our git workflow for working with our Bottlerocket fork while maintaining compatibility with upstream.

## Overview

We maintain a fork of bottlerocket-os/bottlerocket at The-Mines/bottlerocket-mm. This workflow ensures we can:
- Keep our fork in sync with upstream changes
- Develop features independently
- Easily create pull requests to upstream
- Maintain a stable integration branch for our work

## Repository Setup

### Remotes
- **origin**: `https://github.com/bottlerocket-os/bottlerocket.git` (upstream)
- **the-mines**: `https://github.com/The-Mines/bottlerocket-mm.git` (our fork)

### Branch Structure
- **`develop`**: Tracks `origin/develop` (upstream) - NEVER commit directly here
- **`the-mines/main`**: Our main integration branch
- **`feature/*`**: Feature branches for new development

## Initial Setup

If you haven't set up the remotes yet:

```bash
# Clone from upstream
git clone https://github.com/bottlerocket-os/bottlerocket.git
cd bottlerocket

# Add your fork as a remote
git remote add the-mines https://github.com/The-Mines/bottlerocket-mm.git

# Set up branch tracking
git checkout develop
git branch -u origin/develop

# Create and push our main branch
git checkout -b the-mines/main
git push -u the-mines the-mines/main

# Configure push default to prevent accidental pushes to upstream
git config remote.pushdefault the-mines
```

## Daily Workflow

### 1. Starting New Work

Always start from an up-to-date upstream develop:

```bash
# Update from upstream
git checkout develop
git pull origin develop

# Create a feature branch
git checkout -b feature/your-feature-name
```

### 2. Development

Work on your feature branch:

```bash
# Make changes
vim src/some-file.rs

# Commit changes
git add .
git commit -m "feat: add new functionality"

# Push to your fork
git push -u the-mines feature/your-feature-name
```

### 3. Integration

Merge completed features to our main branch:

```bash
# Switch to our main branch
git checkout the-mines/main

# Merge the feature
git merge feature/your-feature-name

# Push to our fork
git push the-mines the-mines/main
```

### 4. Syncing with Upstream

Periodically sync your fork's develop with upstream:

```bash
# Update local develop from upstream
git checkout develop
git pull origin develop

# Push to your fork's develop (optional)
git push the-mines develop
```

## Creating Pull Requests

### To Upstream (bottlerocket-os)

1. Ensure your feature branch is based on latest upstream develop
2. Push your feature branch to your fork
3. Create PR from `the-mines/feature/your-feature` to `bottlerocket-os/develop`

### To Our Fork

1. Push feature branch to the-mines
2. Create PR from `feature/your-feature` to `the-mines/main`

## Common Commands

```bash
# View all branches and their tracking
git branch -vv

# See all remotes
git remote -v

# Update from upstream
git checkout develop && git pull origin develop

# Start new feature
git checkout develop
git pull origin develop
git checkout -b feature/new-feature

# Push to your fork
git push -u the-mines feature/new-feature

# Check what would be pushed
git push --dry-run the-mines

# Fetch all remotes
git fetch --all
```

## Best Practices

### DO:
- ✅ Always start features from updated `develop`
- ✅ Use descriptive branch names: `feature/add-api-endpoint`
- ✅ Keep commits atomic and well-described
- ✅ Regularly sync with upstream
- ✅ Test before merging to `the-mines/main`

### DON'T:
- ❌ Never commit directly to `develop`
- ❌ Don't force push to shared branches
- ❌ Avoid long-lived feature branches
- ❌ Don't push to origin (upstream) by accident

## Troubleshooting

### Accidentally committed to develop

```bash
# Save your work
git stash

# Reset develop to upstream
git checkout develop
git fetch origin
git reset --hard origin/develop

# Create feature branch and apply changes
git checkout -b feature/my-changes
git stash pop
```

### Need to update feature branch with latest develop

```bash
git checkout feature/your-feature
git fetch origin
git rebase origin/develop
```

### Pushed to wrong remote

```bash
# If you pushed to origin by mistake
# Contact upstream maintainers or delete the branch if possible

# Set up push default to prevent this
git config remote.pushdefault the-mines
```

## Example Workflow

Here's a complete example of implementing a new feature:

```bash
# 1. Start fresh
git checkout develop
git pull origin develop

# 2. Create feature branch
git checkout -b feature/platform-api

# 3. Work on the feature
cd platform-control-agent
make test
git add .
git commit -m "feat: implement Platform Control Agent"

# 4. Push to fork
git push -u the-mines feature/platform-api

# 5. Create PR on GitHub
# Visit: https://github.com/The-Mines/bottlerocket-mm
# Create PR to the-mines/main

# 6. After PR approved and merged
git checkout the-mines/main
git pull the-mines the-mines/main

# 7. Clean up
git branch -d feature/platform-api
```

## Platform API Development

For the Platform API project specifically:

```bash
# Working on platform-control-agent
cd platform-control-agent
make init
make run-tests

# Before committing
make fmt
make lint
make test
```

## Questions?

- Check remote configuration: `git remote -v`
- Check branch tracking: `git branch -vv`
- See push destination: `git config remote.pushdefault`

For more help, see the [Git documentation](https://git-scm.com/doc) or ask the team.