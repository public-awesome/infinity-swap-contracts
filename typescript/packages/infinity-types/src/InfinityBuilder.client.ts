/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.30.1.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Decimal, Uint128, InstantiateMsg, CodeIds, Coin, ExecuteMsg, QueryMsg } from "./InfinityBuilder.types";
export interface InfinityBuilderReadOnlyInterface {
  contractAddress: string;
}
export class InfinityBuilderQueryClient implements InfinityBuilderReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
  }

}