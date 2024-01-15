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
  OpenLiquidityPositionData,
  useOpenLiquidityPositionData,
} from "../hooks/OpenLiquidityPositionData";
import {
  ConnectedAccount,
  useConnectedAccounts,
} from "../hooks/ConnectedAccounts";
import { RadixDappToolkit } from "@radixdlt/radix-dapp-toolkit";
import { SUPPORTED_EXCHANGES } from "../constants";
import { dAppToolkit } from "../RadixDappToolkit";
import { useLastTransactionHash } from "../hooks/LastTransactionHash";
import { Dispatch, SetStateAction } from "react";

const NO_X_AXIS_BORDERS_CLASS_NAME: string = "mx-0 px-0";

export const OpenLiquidityPosition = (props: Props) => {
  const [openLiquidityPositionData, setOpenLiquidityPositionData] =
    useOpenLiquidityPositionData();
  const [lastTx, setLastTx] = useLastTransactionHash();

  const connectedAccount = useConnectedAccounts();

  const allDataDefined = !Object.entries(openLiquidityPositionData).some(
    ([_, v]) => v === undefined,
  );
  const isAccountConnected = connectedAccount.status === "success";

  return (
    <div style={props.style}>
      <h3 style={{ marginRight: "auto", fontWeight: 600 }}>
        {props.title || "Provide Liquidity"}
      </h3>

      <Container>
        <Row className="font-weight-bold">
          <Col className={NO_X_AXIS_BORDERS_CLASS_NAME}>
            <h5 className={NO_X_AXIS_BORDERS_CLASS_NAME}>Exchange</h5>
          </Col>
          <Col>
            <h5 className={NO_X_AXIS_BORDERS_CLASS_NAME}>User Resource</h5>
          </Col>
          <Col>
            <h5 className={NO_X_AXIS_BORDERS_CLASS_NAME}>Lockup Period</h5>
          </Col>
          <Col>
            <h5 className={NO_X_AXIS_BORDERS_CLASS_NAME}>Amount</h5>
          </Col>
          <Col></Col>
        </Row>
        <Row>
          <Col className={NO_X_AXIS_BORDERS_CLASS_NAME}>
            <Form.Select
              aria-label="Exchange select"
              defaultValue="Select"
              style={{ borderRadius: "6px 0px 0px 6px" }}
              onChange={(event) => {
                const selectedValue = event.target.value;
                const evaluatedValue =
                  selectedValue === "none" ? undefined : selectedValue;

                // Set the state
                setOpenLiquidityPositionData({
                  ...openLiquidityPositionData,
                  exchange: evaluatedValue,
                });

                // Fire an event
                if (props.onExchangeChange !== undefined) {
                  props.onExchangeChange(evaluatedValue);
                }
              }}
            >
              <option value="none">Select</option>
              {Object.entries(SUPPORTED_EXCHANGES).map(
                ([logicalName, physicalName]) => {
                  return (
                    <option key={logicalName} value={physicalName}>
                      {logicalName}
                    </option>
                  );
                },
              )}
            </Form.Select>
          </Col>
          <Col className={NO_X_AXIS_BORDERS_CLASS_NAME}>
            <Form.Select
              aria-label="User resource select"
              className="rounded-0"
              onChange={(event) => {
                const selectedValue = event.target.value;
                const evaluatedValue =
                  selectedValue === "none" ? undefined : selectedValue;

                // Set the state
                setOpenLiquidityPositionData({
                  ...openLiquidityPositionData,
                  resourceAddress: evaluatedValue,
                });

                // Fire an event
                if (props.onUserResourceChange !== undefined) {
                  props.onUserResourceChange(evaluatedValue);
                }
              }}
            >
              <option value="none">Select</option>
              {Object.entries(bootstrapInformation?.resources || {}).map(
                ([address, information]) => {
                  return (
                    <option key={address} value={address}>
                      {information.name.replace("Fake", "").trim()}
                    </option>
                  );
                },
              )}
            </Form.Select>
          </Col>
          <Col className={NO_X_AXIS_BORDERS_CLASS_NAME}>
            <Form.Select
              aria-label="User lockup period select"
              className="rounded-0"
              onChange={(event) => {
                const selectedValue = event.target.value;
                const evaluatedValue =
                  selectedValue === "none" ? undefined : selectedValue;

                // Set the state
                setOpenLiquidityPositionData({
                  ...openLiquidityPositionData,
                  lockupPeriodSeconds: evaluatedValue,
                });

                // Fire an event
                if (props.onUserResourceChange !== undefined) {
                  props.onUserResourceChange(evaluatedValue);
                }
              }}
            >
              <option value="none">Select</option>
              {Object.entries(
                bootstrapInformation?.protocol.reward_rates || {},
              ).map(([periodInSeconds, percentage]) => {
                return (
                  <option key={periodInSeconds} value={periodInSeconds}>
                    {`${periodInSeconds} seconds - ${
                      parseFloat(percentage) * 100
                    }%`}
                  </option>
                );
              })}
            </Form.Select>
          </Col>
          <Col className={NO_X_AXIS_BORDERS_CLASS_NAME}>
            <Form.Group className="mb-3" controlId="formContributionAmount">
              <Form.Control
                placeholder="Amount"
                className="rounded-0"
                onChange={(event) => {
                  // Remove anything that is not a number
                  let numbers = Array.from({ length: 10 }, (_, i) =>
                    i.toString(),
                  );
                  let string = Array.from(event.target.value).filter((char) =>
                    numbers.includes(char),
                  );

                  // Add the comma separator
                  const evaluatedValue = string.join("");
                  event.target.value = evaluatedValue
                    .toString()
                    .replace(/\B(?=(\d{3})+(?!\d))/g, ",");

                  // Set the state
                  setOpenLiquidityPositionData({
                    ...openLiquidityPositionData,
                    amount: evaluatedValue,
                  });

                  // Fire an event
                  if (props.onAmountChange !== undefined) {
                    props.onAmountChange(evaluatedValue);
                  }
                }}
              />
            </Form.Group>
          </Col>
          <Col className={NO_X_AXIS_BORDERS_CLASS_NAME}>
            <Button
              style={{
                minWidth: "100%",
                borderRadius: "0px 6px 6px 0px",
              }}
              variant="dark"
              disabled={!allDataDefined || !isAccountConnected}
              onClick={(_) => {
                highLevelConstructAndSendManifestToWallet(
                  dAppToolkit,
                  bootstrapInformation,
                  connectedAccount,
                  openLiquidityPositionData,
                  setLastTx,
                );
              }}
            >
              Contribute
            </Button>
          </Col>
        </Row>
      </Container>
    </div>
  );
};

interface Props {
  title?: string;
  style?: React.CSSProperties;
  /* Callbacks */
  onExchangeChange?: (newExchange?: string) => void;
  onUserResourceChange?: (newResourceAddress?: string) => void;
  onAmountChange?: (newAmount?: string) => void;
}

const constructManifest = (
  ignitionAddress: string,
  connectedAccount: string,
  selections: {
    pool: string;
    resourceAddress: string;
    amount: string;
    lockupPeriodInSeconds: string;
  },
): string => {
  return `
    MINT_FUNGIBLE
      Address("${selections.resourceAddress}")
      Decimal("${selections.amount}")
    ;
   
    TAKE_FROM_WORKTOP
      Address("${selections.resourceAddress}")
      Decimal("${selections.amount}")
      Bucket("bucket")
    ;

    CALL_METHOD
      Address("${ignitionAddress}")
      "open_liquidity_position"
      Bucket("bucket")
      Address("${selections.pool}")
      ${selections.lockupPeriodInSeconds}u64
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
  ignitionAddress: string,
  connectedAccount: string,
  selections: {
    pool: string;
    resourceAddress: string;
    amount: string;
    lockupPeriodInSeconds: string;
  },
  setLastTxHash: Dispatch<SetStateAction<string | null>>,
) => {
  const manifest = constructManifest(
    ignitionAddress,
    connectedAccount,
    selections,
  );
  console.log(manifest);
  dAppToolkit.walletApi
    .sendTransaction({ transactionManifest: manifest })
    .map((item) => setLastTxHash(item.transactionIntentHash));
};

const highLevelConstructAndSendManifestToWallet = (
  dAppToolkit: RadixDappToolkit | undefined,
  bootstrapInformation: TestingBootstrapInformation | null,
  connectedAccount: ConnectedAccount,
  selectedInformation: OpenLiquidityPositionData,
  setLastTxHash: Dispatch<SetStateAction<string | null>>,
) => {
  if (
    selectedInformation.amount !== undefined &&
    selectedInformation.exchange !== undefined &&
    selectedInformation.resourceAddress !== undefined &&
    selectedInformation.lockupPeriodSeconds !== undefined &&
    bootstrapInformation !== null &&
    dAppToolkit !== undefined &&
    connectedAccount.status === "success"
  ) {
    const exchangePhysicalName =
      selectedInformation.exchange as keyof TestingBootstrapInformation;

    const exchangeEntries = bootstrapInformation[
      exchangePhysicalName
    ] as DexEntities;

    const ignitionAddress = bootstrapInformation.protocol.ignition;
    const account = connectedAccount.account;

    const poolAddress =
      exchangeEntries.pools[selectedInformation.resourceAddress];

    constructAndSendManifestToWallet(
      dAppToolkit,
      ignitionAddress,
      account,
      {
        pool: poolAddress,
        resourceAddress: selectedInformation.resourceAddress,
        amount: selectedInformation.amount,
        lockupPeriodInSeconds: selectedInformation.lockupPeriodSeconds,
      },
      setLastTxHash,
    );
  } else {
    console.error(
      "Something was undefined when attempting to construct and send the transaction",
    );
  }
};
