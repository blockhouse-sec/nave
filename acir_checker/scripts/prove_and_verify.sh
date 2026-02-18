NAME=$1
bb.js prove_ultra_honk -b ./target/$NAME.json -w ./target/$NAME.gz
bb.js write_vk_ultra_honk -b ./target/$NAME.json -o ./proofs/vk
bb.js verify_ultra_honk -k ./proofs/vk -p ./proofs/proof
echo $?
