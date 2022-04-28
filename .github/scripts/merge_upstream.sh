# Merge upstream into master
git remote add upstream https://github.com/project-serum/anchor.git
git pull upstream master --no-ff --tags
git push origin master
