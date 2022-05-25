# Get versions
anchor_version=$(cat VERSION)
solana_version=$(curl -sL https://api.github.com/repos/solana-labs/solana/releases | jq -r 'map(select(.name | startswith("Testnet"))) | first | .tag_name' | cut -d'v' -f 2)
branch_name=v$anchor_version-solana.$solana_version

# Checkout last released version
git checkout tags/v$anchor_version

# Checkout new branch
git checkout -b $branch_name

# Change Cargo.toml package names
sed -i '' -e "/^name =/s/=.*/= \"cronos-anchor-lang\"/g" ./lang/Cargo.toml
sed -i '' -e "/^name =/s/=.*/= \"cronos-anchor-spl\"/g" ./spl/Cargo.toml

# Update Solana & Anchor dependencies
cargo_tomls=($(find "." -name Cargo.toml))
for cargo_toml in "${cargo_tomls[@]}"; do
    sed -i '' -e "/^solana-/s/=.*/= \"$solana_version\"/g" $cargo_toml
    sed -i '' -e "s/\anchor-spl = {/& package = \"cronos-anchor-spl\",/" $cargo_toml
    sed -i '' -e "s/\anchor-lang = {/& package = \"cronos-anchor-lang\",/" $cargo_toml
done

# Commit changes
git add .
git commit -m "Bump Solana dependencies to v$solana_version" 
git push -f origin $branch_name
