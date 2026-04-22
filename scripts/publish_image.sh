#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="${IMAGE_NAME:-ghcr.io/yral-dapp/yral-backend-dev}"
IMAGE_TAG="${IMAGE_TAG:-${GITHUB_SHA:-local}}"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker CLI is required"
  exit 1
fi

if [[ -n "${GITHUB_ACTOR:-}" && -n "${GITHUB_TOKEN:-}" ]]; then
  echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$GITHUB_ACTOR" --password-stdin
fi

docker build -t "$IMAGE_NAME:$IMAGE_TAG" -f Dockerfile .
docker push "$IMAGE_NAME:$IMAGE_TAG"

echo "Published $IMAGE_NAME:$IMAGE_TAG"
