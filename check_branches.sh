#!/bin/bash
author_name="Koosha Paridehpour"
main_branch="main"

git branch --list | sed 's/[*+ ]//g' | while read branch; do
    if [ "$branch" == "$main_branch" ]; then continue; fi

    # Find the first commit in branch that isn't in main
    first_commit=$(git log $main_branch..$branch --oneline --reverse | head -1 | awk '{print $1}')

    if [ -n "$first_commit" ]; then
        commit_author=$(git log -1 --format='%an' "$first_commit")

        if [ "$commit_author" == "$author_name" ]; then
            # Check for open PR
            pr_info=$(gh pr list --head "$branch" --state open --json number,title,url --jq '.[0]')

            if [ -z "$pr_info" ]; then
                # No open PR found.
                # Get a brief summary of changes
                summary=$(git log -1 --format='%s' "$branch")
                echo "BRANCH: $branch"
                echo "SUMMARY: $summary"
                echo "---"
            fi
        fi
    fi
done
