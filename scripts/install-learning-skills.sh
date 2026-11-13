#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p .agents/skills
for skill in skills/*; do
  name=${skill##*/}
  destination=".agents/skills/$name"
  if [ -e "$destination" ] || [ -L "$destination" ]; then
    if [ "$(readlink "$destination")" = "../../$skill" ]; then continue; fi
    echo "Refusing to replace existing $destination" >&2
    exit 1
  fi
  ln -s "../../$skill" "$destination"
done
