import {
  DataRequestBuilder,
  RadixDappToolkit,
} from "@radixdlt/radix-dapp-toolkit";
import { bootstrapInformation } from "./BootstrapInformation";

export const dAppToolkit: RadixDappToolkit = (() => {
  const dappToolkit = RadixDappToolkit({
    networkId: bootstrapInformation.network_id,
    dAppDefinitionAddress: bootstrapInformation.protocol.dapp_definition,
    applicationName: "Ignition",
    applicationVersion: "1.0.0",
  })
  dappToolkit.walletApi.setRequestData(
    DataRequestBuilder.accounts().exactly(1)
  );
  return dappToolkit
})()
