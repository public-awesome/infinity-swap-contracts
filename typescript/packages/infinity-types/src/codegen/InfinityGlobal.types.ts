/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.30.1.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Decimal = string;
export type Uint128 = string;
export interface InstantiateMsg {
  global_config: GlobalConfigForString;
  min_prices: Coin[];
}
export interface GlobalConfigForString {
  fair_burn: string;
  fair_burn_fee_percent: Decimal;
  infinity_factory: string;
  infinity_index: string;
  infinity_pair_code_id: number;
  infinity_router: string;
  marketplace: string;
  max_royalty_fee_percent: Decimal;
  max_swap_fee_percent: Decimal;
  pair_creation_fee: Coin;
  royalty_registry: string;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface ExecuteMsg {
  [k: string]: unknown;
}
export type QueryMsg = {
  global_config: {};
} | {
  min_price: {
    denom: string;
  };
};
export type Addr = string;
export interface GlobalConfigForAddr {
  fair_burn: Addr;
  fair_burn_fee_percent: Decimal;
  infinity_factory: Addr;
  infinity_index: Addr;
  infinity_pair_code_id: number;
  infinity_router: Addr;
  marketplace: Addr;
  max_royalty_fee_percent: Decimal;
  max_swap_fee_percent: Decimal;
  pair_creation_fee: Coin;
  royalty_registry: Addr;
}
export type NullableCoin = Coin | null;