# Git config
git config --global user.email "elias@cronos.so"
git config --global user.name "eliascm17"

# Merge upstream into master
git remote add upstream https://github.com/project-serum/anchor.git
git pull upstream master --no-ff
git push origin master
