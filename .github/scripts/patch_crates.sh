# source this file

update_solana_dependencies() {
  echo $project_root
  echo $PWD
  echo "$(ls -l)"
  declare project_root="$1"
  declare solana_ver="$2"
  declare tomls=()
  while IFS='' read -r line; do tomls+=("$line"); done < <(find "$project_root" -name Cargo.toml)


  sed -i -e "s#\(solana-program = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-program-test = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-sdk = \"\).*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-sdk = { version = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-client = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-client = { version = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-clap-utils = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-clap-utils = { version = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-cli-config = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-cli-config = { version = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-account-decoder = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-account-decoder = { version = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-faucet = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
  sed -i -e "s#\(solana-faucet = { version = \"\)[^\"]*\(\"\)#\1$solana_ver\2#g" "${tomls[@]}" || return $?
}
