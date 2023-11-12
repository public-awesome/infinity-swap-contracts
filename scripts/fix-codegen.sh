#!/bin/bash

# Navigate to the client/types directory
cd client/types

# Use a loop to process each .ts file
for file in *.ts; do
  # Use sed to replace 'QueryOptions_for_uint64' with 'QueryOptionsForUint64'
  sed -i '' 's/QueryOptions_for_uint64/QueryOptionsForUint64/g' "$file"
done

echo "Replacement complete."
