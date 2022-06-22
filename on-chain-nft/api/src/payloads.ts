export type TokenMetadata = {
    name: string,
    description: string,
    media: string,
    reference: string,
};

export const payloads = {
    init: function(
        name: string,
        symbol: string,
        base_uri: string,
        base_image: string,
        layers: Map<string, Array<string>>,
        royalties: any,
    ) {
        return {
            name,
            symbol,
            base_uri,
            base_image,
            layers,
            royalties,
        }
    },
    mint: function(token_metadata: TokenMetadata, description: Array<number>) {
        return {
            Mint: {
                token_metadata,
                description,
            }
        }
    },
    burn: function(token_id: number) {
        return {
            Burn: {
                token_id,
            }
        }
    },
    transfer: function(to: string, token_id: number) {
        return {
            Transfer: {
                to,
                token_id,
            }
        }
    },
    approve: function(to: string, token_id: number) {
        return {
            Approve: {
                to,
                token_id,
            }
        }
    },
    transferPayout: function(to: string, token_id: number, amount: number) {
        return {
            TransferPayout: {
                to,
                token_id,
                amount,
            }
        }
    }
};