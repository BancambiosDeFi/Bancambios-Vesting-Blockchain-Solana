#!/bin/bash
readonly cluster="localhost"   #(localhost, Devnet, Testnet, Mainnet Beta)
readonly supply=1000000000
readonly disable_minting=false
readonly new_keypair=true
readonly vesting_name=true
readonly airdrop=true

cd token-vesting
out=$(solana config get | awk '{print $3}')
initial_keypair=$(echo $out | awk '{print $4}')
secret_key=$(cat ${initial_keypair})

echo "Setting endpoint..."
if [[ $cluster = "localhost" ]]
then
    exist=$(pidof solana-test-validator)
    if [[ $exist = "" ]]
    then
        echo -e "Can not find running solana test validator\nTo start it, you can use solana-test-validator command"
        exit
    fi
    connection_endpoint="http://localhost:8899"
elif [[ $cluster = "Devnet" ]]
then
    connection_endpoint="https://api.devnet.solana.com"
elif [[ $cluster = "Testnet" ]]
then
    connection_endpoint="https://api.testnet.solana.com"
elif [[ $cluster = "Mainnet Beta" ]]
then
    connection_endpoint="https://api.mainnet-beta.solana.com"
else
    echo "wrong cluster"
    exit
fi

solana config set --url ${connection_endpoint} > /dev/null 2>&1
echo "Endpoint setting complete!"
if $new_keypair
then
    echo "Creating new account..."
    solana-keygen new --force --no-passphrase -so "token-owner.json" > /dev/null 2>&1
    solana config set --keypair token-owner.json > /dev/null 2>&1
    secret_key=$(cat token-owner.json)
    echo "Account creating complete!"
fi

if $airdrop
then
    echo "Airdropping SOL to account..."
    # Airdropping more that 2 SOL is forbidden on Devnet
    solana airdrop 2 > /dev/null 2>&1
    solana airdrop 2 > /dev/null 2>&1
    echo "Airdropping complete!"
fi

rm dist/program/token_vesting-keypair.json > /dev/null 2>&1

echo "Building and deploying token-vesting program..."
bash build.sh  > /dev/null 2>&1
program_id=$(bash deploy.sh | awk '{print $3}')
program_id=$(solana program show $program_id | awk '{print $3}')
program_id=$(echo $program_id  | awk '{print $1}')
echo "Deployment complete!"

echo "Creating new token..."
out=$(spl-token create-token)
token=$(echo "${out}" | awk '{print $3}')
echo "Token creating complete!"

echo "Creating account for token..."
account=$(spl-token create-account $token | awk '{print $3}')
spl-token mint $token $supply > /dev/null 2>&1
echo "Account creating complete!"

if $disable_minting
then
    echo "Disabling mint for token..."
    spl-token authorize $token mint --disable  > /dev/null 2>&1
    echo "Mint disabling complete!"
fi

if $new_keypair
then
    solana config set --keypair $initial_keypair > /dev/null 2>&1
fi

echo "Creating file environment..."
cd ..
exec 6>&1
exec 1>token-vesting-cli/.env
echo "SENDER_SECRET_KEY=\"$secret_key\""
echo "CONNECTION_ENDPOINT=\"$connection_endpoint\""
echo "VESTING_NAME=\"$vesting_name\""
echo "TOKEN_MINT=\"$token\""
echo "VESTING_PROGRAM_ID=\"$program_id\""
echo "REQUEST_AIRDROP=\"$airdrop\""
exec 1>&6 6>&-
echo "Environment file creation complete!"

echo "Building token-vesting-api..."
cd token-vesting-api
sudo yarn install --ignore-scripts > /dev/null 2>&1
tsc > /dev/null 2>&1
sudo yarn link > /dev/null 2>&1
echo "Building complete!"
echo "Building token-vesting-cli..."
cd ../token-vesting-cli > /dev/null 2>&1
sudo yarn install --ignore-scripts > /dev/null 2>&1
sudo yarn link token-vesting-api > /dev/null 2>&1
tsc > /dev/null 2>&1
sudo yarn start > /dev/null 2>&1
echo "Building complete!"
cd ..
cd token-vesting
echo "Information about token:"
spl-token account-info $token

tv --help