START_DIR=$(pwd)

for d in contracts/*; do
  if [ -d "$d" ]; then
    cd $d
    cargo schema
    rm -rf schema/raw
    cd "$START_DIR"
  fi
done