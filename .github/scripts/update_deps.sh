# Get versions
anchor_version=$(cat VERSION)
solana_version=$(curl -sL https://api.github.com/repos/solana-labs/solana/releases | jq -r 'map(select(.name | startswith("Testnet"))) | first | .tag_name' | cut -d'v' -f 2)
branch_name=v$anchor_version-solana.$solana_version

# Checkout last released version
git checkout tags/v$anchor_version

# Checkout new branch
git checkout -b $branch_name

# Update Solana dependencies
tomls=($(find "." -name Cargo.toml))
sed -i -e "s#\(solana-program = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-program-test = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-sdk = \"\).*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-sdk = { version = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-client = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-client = { version = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-clap-utils = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-clap-utils = { version = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-cli-config = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-cli-config = { version = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-account-decoder = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-account-decoder = { version = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-faucet = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"
sed -i -e "s#\(solana-faucet = { version = \"\)[^\"]*\(\"\)#\1=$solana_version\2#g" "${tomls[@]}"

# Commit changes
git add .
git commit -m "Bump Solana dependencies to v$solana_version" 
git push -f origin $branch_name
