// @flow

import { remote, shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { goBack } from 'connected-react-router';
import Account from '../components/Account';
import { links } from '../../config';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  accountToken: state.account.accountToken,
  accountExpiry: state.account.expiry,
  expiryLocale: remote.app.getLocale(),
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const history = bindActionCreators({ goBack }, dispatch);
  return {
    onLogout: () => {
      props.app.logout();
    },
    onClose: () => {
      history.goBack();
    },
    onBuyMore: () => shell.openExternal(links['purchase']),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Account);
