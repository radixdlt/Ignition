import Container from "react-bootstrap/Container";
import Row from "react-bootstrap/Row";
import Col from "react-bootstrap/Col";
import Form from "react-bootstrap/Form";
import { Button } from "react-bootstrap";

import {
  DexEntities,
  TestingBootstrapInformation,
  bootstrapInformation,
} from "../BootstrapInformation";
import {
  ConnectedAccount,
  useConnectedAccounts,
} from "../hooks/ConnectedAccounts";
import { RadixDappToolkit } from "@radixdlt/radix-dapp-toolkit";
import { SUPPORTED_EXCHANGES } from "../constants";
import { dAppToolkit } from "../RadixDappToolkit";
import { Dispatch, SetStateAction } from "react";

const NO_X_AXIS_BORDERS_CLASS_NAME: string = "mx-0 px-0";

export const MintUtility = (props: Props) => {
  const connectedAccount = useConnectedAccounts();

  const isAccountConnected = connectedAccount.status === "success";

  if (connectedAccount.status === "success") {
    return (
      <div style={props.style}>
        <h3 style={{ marginRight: "auto", fontWeight: 600 }}>
          {props.title || "Mint Utility"}
        </h3>

        <div className="d-flex flex-row justify-content-center">
          {[
            [bootstrapInformation.protocol.protocol_resource, "XRD"],
            ...Object.entries(bootstrapInformation.resources).map(
              ([address, information]) => [address, information.symbol],
            ),
          ].map(([resourceAddress, symbol]) => (
            <Button
              className="mx-3"
              variant="dark"
              onClick={(_) =>
                constructAndSendManifestToWallet(
                  connectedAccount.account,
                  resourceAddress,
                )
              }
            >
              Mint {symbol}
            </Button>
          ))}
        </div>
      </div>
    );
  } else {
    return (
      <div style={props.style}>
        <h3 style={{ marginRight: "auto", fontWeight: 600 }}>
          {props.title || "Mint Utility"}
        </h3>

        <div className="d-flex flex-row justify-content-center">
          {[
            [bootstrapInformation.protocol.protocol_resource, "XRD"],
            ...Object.entries(bootstrapInformation.resources).map(
              ([address, information]) => [address, information.symbol],
            ),
          ].map(([resourceAddress, symbol]) => (
            <Button className="mx-3" variant="dark" disabled={true}>
              Mint {symbol}
            </Button>
          ))}
        </div>
      </div>
    );
  }
};

interface Props {
  title?: string;
  style?: React.CSSProperties;
}

const constructManifest = (
  connectedAccount: string,
  resourceAddress: string,
): string => {
  return `
    MINT_FUNGIBLE
      Address("${resourceAddress}")
      Decimal("100000000000")
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
  connectedAccount: string,
  resourceAddress: string,
) => {
  const manifest = constructManifest(connectedAccount, resourceAddress);
  console.log(manifest);
  dAppToolkit.walletApi.sendTransaction({ transactionManifest: manifest });
};
