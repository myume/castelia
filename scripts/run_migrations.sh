#!/usr/bin/env bash

for dir in $(find . -type d -name "migrations"); do
        echo "Applying migrations in $dir..."
        sqlx migrate run --source "$dir" --ignore-missing
done
