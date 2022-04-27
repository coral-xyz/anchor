
update_solana_dependencies() {
  echo $PWD
}

git config --global user.email "nick@cronos.so"
git config --global user.name "nickgarfield"
git remote add upstream https://github.com/project-serum/anchor.git
git pull upstream master --no-ff
git push origin master

//

git checkout -b nick-mybranch
git merge master

solana_version=$(curl -sL https://api.github.com/repos/solana-labs/solana/releases | jq -r 'map(select(.name | startswith("Testnet"))) | first | .tag_name')
solana_ver_cut=$(echo $solana_ver | cut -d'v' -f 2)

update_solana_dependencies . $solana_ver_cut

git commit -m 'updating solana deps'


