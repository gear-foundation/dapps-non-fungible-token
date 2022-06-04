const { GearApi, GearKeyring, getWasmMetadata } = require("@gear-js/api");
const { readFileSync } = require('fs');
const path = require('path');
const fs = require('fs');


require('dotenv').config();

const MINT_COUNT = 10;

async function main() {
    const gearApi = await GearApi.create();
    const jsonKeyring = readFileSync('./account.json').toString();;
    const account = GearKeyring.fromJson(jsonKeyring, 'Google06!!');
    const metaFile = readFileSync(process.env.META_WASM);
    const meta = metaFile ? await getWasmMetadata(metaFile) : undefined;
    console.log(process.env.PROGRAM_ID)
    let lc = getLayersCount("./cryptopunk-nft-generator/layers_svg/trait_types");
    let cartesianCombinations = lc[0];
    for(let i = 1; i < lc.length; i++) {
        cartesianCombinations = cartesian(cartesianCombinations, lc[i]);
    };
    for(let i = 0; i < MINT_COUNT; i++) {
        try {
            let desc = {};
            let combination = cartesianCombinations[i];
            combination.map((el, i) => {
                desc[parseInt(i)] = parseInt(el);
            });
            console.log(desc);
            let somePayload = {
                Mint: {
                    token_metadata: {
                        name: `COC #${i}`,
                        description: `Cryptopunk number ${i}`,
                        media: `http://cryptopunk.com/media/${i}`,
                        reference: `http://cryptopunk.com/media/${i}`,
                    }
                }
            }

            const gas = await gearApi.program.gasSpent.handle(
                account.publicKey,
                process.env.PROGRAM_ID,
                somePayload,
                0,
                meta,
            );
            console.log("GAS", gas.toHuman());

            const message = {
                destination: process.env.PROGRAM_ID, // programId
                payload: somePayload,
                gasLimit: gas,
                value: 0,
            };
            // In that case payload will be encoded using meta.handle_input type
            await gearApi.message.submit(message, meta);
            // So if you want to use another type you can specify it
            // await gearApi.message.submit(message, meta, meta.async_handle_input); // For example
        } catch (error) {
            console.error(`${error.name}: ${error.message}`);
        }
        try {
            await gearApi.message.signAndSend(account, (event) => {
                console.log("EVENT", event.toHuman());
            });
        } catch (error) {
            console.error(`${error.name}: ${error.message}`);
        }
    }
}

main();
