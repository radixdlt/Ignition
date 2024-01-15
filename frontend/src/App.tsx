import "./App.css";
import { CloseLiquidityPosition } from "./components/CloseLiquidityPosition";

import { OpenLiquidityPosition } from "./components/OpenLiquidityPosition";
import { useConnectedAccounts } from "./hooks/ConnectedAccounts";

declare global {
  namespace JSX {
    interface IntrinsicElements {
      "radix-connect-button": React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement>,
        HTMLElement
      >;
      "radix-dapps-dropdown": React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement>,
        HTMLElement
      >;
      "radix-tabs-menu": React.DetailedHTMLProps<
        React.HTMLAttributes<HTMLElement>,
        HTMLElement
      >;
    }
  }
}

function App() {
  useConnectedAccounts();

  return (
    <>
      <div
        style={{
          zIndex: 2,
          position: "absolute",
          right: 30,
          top: 30,
        }}
      >
        <radix-connect-button />
      </div>
      <div id="ignition-root" className="iosevka">
        <h1 className="iosevka" style={{ fontSize: 48, fontWeight: 700 }}>
          🔥 Ignition
        </h1>
        <p className="iosevka">
          Double your liquidity contributions and get protection from
          impermanent loss.
        </p>

        {/* Main Body */}
        <div style={{ minWidth: 900, width: 900, maxWidth: 900 }}>
          <OpenLiquidityPosition
            style={{ width: "100%" }}
            title="Open Liquidity Position"
          />
          <CloseLiquidityPosition title="Close Liquidity Position" />
        </div>
      </div>
    </>
  );
}

export default App;
