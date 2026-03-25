#!/bin/bash
author_name="Koosha Paridehpour"
main_branch="main"
open_pr_branches="$1"

git branch --list | sed 's/[*+ ]//g' | while read branch; do
    if [ "$branch" == "$main_branch" ]; then continue; fi

    # Check if this branch is in the open_pr_branches list
    found=0
    for pr_branch in $open_pr_branches; do
        if [ "$pr_branch" == "$branch" ]; then
            found=1
            break
        fi
    done

    if [ "$found" -eq 1 ]; then continue; fi

    # Find the first commit in branch that isn't in main
    first_commit=$(git log $main_branch..$branch --oneline --reverse | head -1 | awk '{print $1}')

    if [ -n "$first_commit" ]; then
        commit_author=$(git log -1 --format='%an' "$first_commit")

        if [ "$commit_author" == "$author_name" ]; then
            # No open PR found.
            # Get a brief summary of changes
            summary=$(git log -1 --format='%s' "$branch")
            echo "BRANCH: $branch"
            echo "SUMMARY: $summary"
            echo "---"
        fi
    fi
done
