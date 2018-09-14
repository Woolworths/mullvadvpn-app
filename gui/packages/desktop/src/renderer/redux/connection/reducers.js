// @flow

import type { ReduxAction } from '../store';
import type { AfterDisconnect, BlockReason, TunnelState, Ip } from '../../lib/daemon-rpc';

export type ConnectionReduxState = {
  status: TunnelState,
  isOnline: boolean,
  ip: ?Ip,
  latitude: ?number,
  longitude: ?number,
  country: ?string,
  city: ?string,
  afterDisconnect: ?AfterDisconnect,
  blockReason: ?BlockReason,
};

const initialState: ConnectionReduxState = {
  status: 'disconnected',
  isOnline: true,
  ip: null,
  latitude: null,
  longitude: null,
  country: null,
  city: null,
  afterDisconnect: null,
  blockReason: null,
};

export default function(
  state: ConnectionReduxState = initialState,
  action: ReduxAction,
): ConnectionReduxState {
  switch (action.type) {
    case 'NEW_LOCATION':
      return { ...state, ...action.newLocation };

    case 'CONNECTING':
      return { ...state, ...{ status: 'connecting', afterDisconnect: null, blockReason: null } };

    case 'CONNECTED':
      return { ...state, ...{ status: 'connected', afterDisconnect: null, blockReason: null } };

    case 'DISCONNECTED':
      return { ...state, ...{ status: 'disconnected', afterDisconnect: null, blockReason: null } };

    case 'DISCONNECTING':
      return {
        ...state,
        ...{ status: 'disconnecting', afterDisconnect: action.afterDisconnect, blockReason: null },
      };

    case 'BLOCKED':
      return {
        ...state,
        ...{ status: 'blocked', afterDisconnect: null, blockReason: action.reason },
      };

    case 'ONLINE':
      return { ...state, ...{ isOnline: true } };

    case 'OFFLINE':
      return { ...state, ...{ isOnline: false } };

    default:
      return state;
  }
}
