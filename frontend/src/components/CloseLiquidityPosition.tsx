import {
  ProgrammaticScryptoSborValueTuple,
  RadixDappToolkit,
  StateNonFungibleDetailsResponseItem,
} from "@radixdlt/radix-dapp-toolkit";
import { SUPPORTED_EXCHANGES } from "../constants";
import {
  DexEntities,
  TestingBootstrapInformation,
  bootstrapInformation,
} from "../BootstrapInformation";
import { useConnectedAccounts } from "../hooks/ConnectedAccounts";
import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { Button } from "react-bootstrap";
import { dAppToolkit } from "../RadixDappToolkit";
import { useLastTransactionHash } from "../hooks/LastTransactionHash";

export const CloseLiquidityPosition = (props: Props) => {
  // Map<address, Map<local_id, data>>
  const [state, setState] = useState<
    Record<string, Record<string, LiquidityReceipt>>
  >({});
  const [lastTx, setLastTx] = useLastTransactionHash();

  // Getting the state
  const configuration = bootstrapInformation;
  const connectedAccount = useConnectedAccounts();

  useEffect(() => {
    if (connectedAccount.status === "success") {
      const accountAddress = connectedAccount.account;

      const liquidityReceiptResources = Object.entries(SUPPORTED_EXCHANGES).map(
        ([logicalName, physicalName]): [string, string] => [
          (
            configuration[
              physicalName as keyof TestingBootstrapInformation
            ] as DexEntities
          ).receipt_resource,
          logicalName,
        ],
      );

      dAppToolkit.gatewayApi.state
        .getEntityDetailsVaultAggregated(accountAddress)
        .then((response) => response.non_fungible_resources.items)
        .then(async (nonFungibles) => {
          // Map<address, Map<local_id, data>>
          let record: Record<string, Record<string, LiquidityReceipt>> = {};

          const liquidityReceiptResourceAddresses =
            liquidityReceiptResources.map(([address, _]) => address);
          for (const item of nonFungibles) {
            const resourceAddress = item.resource_address;
            if (liquidityReceiptResourceAddresses.includes(resourceAddress)) {
              const localIds = item.vaults.items
                .map((item) => item.items || [])
                .flat();

              // Getting the data of the non-fungibles
              const nonFungibleData =
                await dAppToolkit.gatewayApi.state.getNonFungibleData(
                  resourceAddress,
                  localIds,
                );

              const data: Record<string, LiquidityReceipt> = {};

              for (const nftData of nonFungibleData) {
                const localId = nftData.non_fungible_id;

                const receipt: LiquidityReceipt = {
                  name: getValue("name", nftData),
                  description: getValue("description", nftData),
                  keyImageUrl: getValue("key_image_url", nftData),
                  lockupPeriod: getValue("lockup_period", nftData),
                  redemptionUrl: getValue("redemption_url", nftData),
                  poolAddress: getValue("pool_address", nftData),
                  userResourceAddress: getValue(
                    "user_resource_address",
                    nftData,
                  ),
                  userContributionAmount: getValue(
                    "user_contribution_amount",
                    nftData,
                  ),
                  protocolContributionAmount: getValue(
                    "protocol_contribution_amount",
                    nftData,
                  ),
                  maturityTimestamp: getValue("maturity_date", nftData),
                };

                data[localId] = receipt;
              }

              record[resourceAddress] = data;
            }
          }

          return record;
        })
        .then(setState);
    }
  }, [connectedAccount, configuration, lastTx]);

  if (connectedAccount.status === "success") {
    return (
      <div style={props.style}>
        <h3 style={{ marginRight: "auto", fontWeight: 600 }}>
          {props.title || "Remove Liquidity"}
        </h3>

        <table style={{ width: "100%" }}>
          <thead>
            <tr>
              <th>Exchange</th>
              <th>Resource</th>
              <th>User Contribution</th>
              <th>Maturity Date</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {Object.entries(state).map(([liquidityResourceAddress, data]) =>
              Object.entries(data).map(([localId, receiptData]) => {
                return (
                  <tr key={`${liquidityResourceAddress}:${localId}`}>
                    <td>
                      {
                        Object.fromEntries(
                          Object.entries(SUPPORTED_EXCHANGES).map(
                            ([logicalName, physicalName]): [string, string] => [
                              (
                                configuration[
                                  physicalName as keyof TestingBootstrapInformation
                                ] as DexEntities
                              ).receipt_resource,
                              logicalName,
                            ],
                          ),
                        )[liquidityResourceAddress]
                      }
                    </td>
                    <td>
                      {
                        configuration.resources[receiptData.userResourceAddress]
                          .name
                      }
                    </td>
                    <td>{receiptData.userContributionAmount}</td>
                    <td>
                      {new Date(
                        parseInt(receiptData.maturityTimestamp) * 1000,
                      ).toLocaleString()}
                    </td>
                    <td>
                      <Button
                        style={{
                          minWidth: "100%",
                        }}
                        variant="dark"
                        onClick={(_) => {
                          constructAndSendManifestToWallet(
                            dAppToolkit,
                            connectedAccount.account,
                            bootstrapInformation.protocol.ignition,
                            liquidityResourceAddress,
                            localId,
                            setLastTx,
                          );
                        }}
                      >
                        Close Position
                      </Button>
                    </td>
                  </tr>
                );
              }),
            )}
          </tbody>
        </table>
      </div>
    );
  } else {
    return (
      <div style={props.style}>
        <h3 style={{ marginRight: "auto", fontWeight: 600 }}>
          {props.title || "Remove Liquidity"}
        </h3>
      </div>
    );
  }
};

interface Props {
  title?: string;
  style?: React.CSSProperties;
  /* Callbacks */
  onExchangeChange?: (newExchange?: string) => void;
  onUserResourceChange?: (newResourceAddress?: string) => void;
  onAmountChange?: (newAmount?: string) => void;
}

interface LiquidityReceipt {
  name: string;
  description: string;
  keyImageUrl: string;
  lockupPeriod: string;
  redemptionUrl: string;
  poolAddress: string;
  userResourceAddress: string;
  userContributionAmount: string;
  protocolContributionAmount: string;
  maturityTimestamp: string;
}

const getValue = (
  fieldName: string,
  value: StateNonFungibleDetailsResponseItem,
): string => {
  // @ts-ignore
  return (
    value.data?.programmatic_json as ProgrammaticScryptoSborValueTuple
  ).fields.find((item) => item.field_name === fieldName)["value"];
};

const constructManifest = (
  connectedAccount: string,
  ignitionAddress: string,
  liquidityReceipt: string,
  localId: string,
): string => {
  return `
    CALL_METHOD
      Address("${connectedAccount}")
      "withdraw_non_fungibles"
      Address("${liquidityReceipt}")
      Array<NonFungibleLocalId>(
        NonFungibleLocalId("${localId}")
      )
    ;
   
    TAKE_NON_FUNGIBLES_FROM_WORKTOP
      Address("${liquidityReceipt}")
      Array<NonFungibleLocalId>(
        NonFungibleLocalId("${localId}")
      )
      Bucket("bucket")
    ;

    CALL_METHOD
      Address("${ignitionAddress}")
      "close_liquidity_position"
      Bucket("bucket")
    ;

    CALL_METHOD
      Address("${connectedAccount}")
      "try_deposit_batch_or_abort"
      Expression("ENTIRE_WORKTOP")
      None
    ;
  `;
};

const constructAndSendManifestToWallet = (
  dAppToolkit: RadixDappToolkit,
  connectedAccount: string,
  ignitionAddress: string,
  liquidityReceipt: string,
  localId: string,
  setLastTxHash: Dispatch<SetStateAction<string | null>>,
) => {
  const manifest = constructManifest(
    connectedAccount,
    ignitionAddress,
    liquidityReceipt,
    localId,
  );
  console.log(manifest);
  dAppToolkit.walletApi
    .sendTransaction({ transactionManifest: manifest })
    .map((item) => setLastTxHash(item.transactionIntentHash));
};
