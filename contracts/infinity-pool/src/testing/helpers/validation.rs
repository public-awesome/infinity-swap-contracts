use cosmwasm_std::Uint128;

use crate::swap_processor::{NftPayment, TokenPayment};
use crate::testing::helpers::msg::NftSaleCheckParams;

pub fn check_nft_sale(scp: NftSaleCheckParams) {
    assert_eq!(scp.swaps[0].spot_price.u128(), scp.expected_spot_price);
    let expected_nft_payment = Some(NftPayment {
        nft_token_id: scp.token_id,
        address: scp.expected_nft_payer.to_string(),
    });
    assert_eq!(scp.swaps[0].nft_payment, expected_nft_payment);

    let network_fee = scp.swaps[0].network_fee.u128();

    assert_eq!(network_fee, scp.expected_network_fee);
    let mut expected_price = scp.expected_spot_price - network_fee;
    expected_price -= scp.expected_royalty_price;
    expected_price -= scp.expected_finders_fee;

    let expected_seller_payment = Some(TokenPayment {
        amount: Uint128::new(expected_price),
        address: scp.expected_seller.to_string(),
    });
    assert_eq!(scp.swaps[0].seller_payment, expected_seller_payment);

    let expected_finder_payment = match scp.expected_finders_fee {
        0 => None,
        _ => Some(TokenPayment {
            amount: Uint128::from(scp.expected_finders_fee),
            address: scp.expected_finder.to_string(),
        }),
    };
    assert_eq!(scp.swaps[0].finder_payment, expected_finder_payment);

    let expected_royalty_payment = Some(TokenPayment {
        amount: Uint128::new(scp.expected_royalty_price),
        address: scp.creator.to_string(),
    });
    assert_eq!(scp.swaps[0].royalty_payment, expected_royalty_payment);
}
