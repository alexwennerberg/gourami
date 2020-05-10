set -e 

cargo build --release
ansible-playbook playbook.yml

