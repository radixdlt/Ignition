import config from "./config.json"

export const bootstrapInformation: TestingBootstrapInformation = config;

export interface TestingBootstrapInformation {
  network_id: number;
  resources: Record<string, ResourceInformation>;
  protocol: ProtocolConfiguration;
  caviarnine: DexEntities;
  ociswap: DexEntities;
}

export interface ProtocolConfiguration {
  ignition_package_address: string;
  ignition: string;
  protocol_resource: string;
  oracle_package_address: string;
  oracle: string;
  dapp_definition: string;
  reward_rates: Record<string, string>
}

export interface DexEntities {
  package: string;
  pools: Record<string, string>;
  adapter_package: string;
  adapter: string;
  receipt_resource: string;
}

export interface ResourceInformation {
  divisibility: number;
  name: string;
  symbol: string;
  icon_url: string;
}
