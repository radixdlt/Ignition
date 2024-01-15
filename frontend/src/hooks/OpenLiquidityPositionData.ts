import { useState } from "react";

export const useOpenLiquidityPositionData = () => {
  return useState<OpenLiquidityPositionData>({
    exchange: undefined,
    resourceAddress: undefined,
    amount: undefined,
    lockupPeriodSeconds: undefined
  });
};

export interface OpenLiquidityPositionData {
  exchange: string | undefined;
  resourceAddress: string | undefined;
  amount: string | undefined;
  lockupPeriodSeconds: string | undefined
}
