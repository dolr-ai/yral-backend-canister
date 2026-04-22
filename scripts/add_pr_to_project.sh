#!/usr/bin/env bash
set -euo pipefail

ORGANIZATION="${ORGANIZATION:-${GITHUB_REPOSITORY_OWNER:-}}"
PROJECT_NUMBER="${PROJECT_NUMBER:-12}"
PR_ID="${PR_ID:-}"
STATUS_NAME="${STATUS_NAME:-Status}"
STATUS_OPTION_NAME="${STATUS_OPTION_NAME:-Todo}"

if [[ -z "${GH_TOKEN:-}" ]]; then
  echo "GH_TOKEN is required"
  exit 1
fi

if [[ -z "$ORGANIZATION" ]]; then
  echo "ORGANIZATION or GITHUB_REPOSITORY_OWNER is required"
  exit 1
fi

if [[ -z "$PR_ID" ]]; then
  echo "PR_ID is required"
  exit 1
fi

project_json=$(gh api graphql -f query='query($org: String!, $number: Int!) { organization(login: $org){ projectV2(number: $number) { id fields(first:20) { nodes { ... on ProjectV2Field { id name } ... on ProjectV2SingleSelectField { id name options { id name } } } } } } }' -f org="$ORGANIZATION" -F number="$PROJECT_NUMBER")

project_id=$(echo "$project_json" | jq -r '.data.organization.projectV2.id')
status_field_id=$(echo "$project_json" | jq -r --arg name "$STATUS_NAME" '.data.organization.projectV2.fields.nodes[] | select(.name == $name) | .id' | head -n1)
status_option_id=$(echo "$project_json" | jq -r --arg field "$STATUS_NAME" --arg option "$STATUS_OPTION_NAME" '.data.organization.projectV2.fields.nodes[] | select(.name == $field) | .options[] | select(.name == $option) | .id' | head -n1)

if [[ -z "$project_id" || "$project_id" == "null" ]]; then
  echo "Project id not found"
  exit 1
fi

if [[ -z "$status_field_id" || "$status_field_id" == "null" ]]; then
  echo "Status field id not found"
  exit 1
fi

if [[ -z "$status_option_id" || "$status_option_id" == "null" ]]; then
  echo "Status option id not found"
  exit 1
fi

item_id=$(gh api graphql -f query='mutation($project:ID!, $pr:ID!) { addProjectV2ItemById(input: {projectId: $project, contentId: $pr}) { item { id } } }' -f project="$project_id" -f pr="$PR_ID" --jq '.data.addProjectV2ItemById.item.id')

gh api graphql -f query='mutation ($project: ID!, $item: ID!, $status_field: ID!, $status_value: String!) { set_status: updateProjectV2ItemFieldValue(input: { projectId: $project itemId: $item fieldId: $status_field value: { singleSelectOptionId: $status_value } }) { projectV2Item { id } } }' -f project="$project_id" -f item="$item_id" -f status_field="$status_field_id" -f status_value="$status_option_id" --silent

echo "Updated project item $item_id"
