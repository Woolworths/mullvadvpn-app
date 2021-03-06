// @flow

import type { ReduxAction } from '../store';
import type { RelayProtocol, RelayLocation } from '../../lib/daemon-rpc';

export type RelaySettingsRedux =
  | {|
      normal: {
        location: 'any' | RelayLocation,
        port: 'any' | number,
        protocol: 'any' | RelayProtocol,
      },
    |}
  | {|
      customTunnelEndpoint: {
        host: string,
        port: number,
        protocol: RelayProtocol,
      },
    |};

export type RelayLocationRelayRedux = {
  hostname: string,
  ipv4AddrIn: string,
  ipv4AddrExit: string,
  includeInCountry: boolean,
  weight: number,
};

export type RelayLocationCityRedux = {
  name: string,
  code: string,
  latitude: number,
  longitude: number,
  hasActiveRelays: boolean,
  relays: Array<RelayLocationRelayRedux>,
};

export type RelayLocationRedux = {
  name: string,
  code: string,
  hasActiveRelays: boolean,
  cities: Array<RelayLocationCityRedux>,
};

export type SettingsReduxState = {
  relaySettings: RelaySettingsRedux,
  relayLocations: Array<RelayLocationRedux>,
  autoConnect: boolean,
  allowLan: boolean,
  enableIpv6: boolean,
};

const initialState: SettingsReduxState = {
  relaySettings: {
    normal: {
      location: 'any',
      port: 'any',
      protocol: 'any',
    },
  },
  relayLocations: [],
  autoConnect: false,
  allowLan: false,
  enableIpv6: true,
};

export default function(
  state: SettingsReduxState = initialState,
  action: ReduxAction,
): SettingsReduxState {
  switch (action.type) {
    case 'UPDATE_RELAY':
      return {
        ...state,
        relaySettings: action.relay,
      };

    case 'UPDATE_RELAY_LOCATIONS':
      return {
        ...state,
        relayLocations: action.relayLocations,
      };

    case 'UPDATE_ALLOW_LAN':
      return {
        ...state,
        allowLan: action.allowLan,
      };

    case 'UPDATE_AUTO_CONNECT':
      return {
        ...state,
        autoConnect: action.autoConnect,
      };

    case 'UPDATE_ENABLE_IPV6':
      return {
        ...state,
        enableIpv6: action.enableIpv6,
      };

    default:
      return state;
  }
}
