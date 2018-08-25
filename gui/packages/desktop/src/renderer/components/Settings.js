// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import Img from './Img';
import { Layout, Container } from './Layout';
import NavigationBar, { CloseBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import CustomScrollbars from './CustomScrollbars';
import styles from './SettingsStyles';
import WindowStateObserver from '../lib/window-state-observer';
import { colors } from '../../config';

/*:: import type { LoginState } from '../redux/account/reducers';*/
/*:: type Props = {
  loginState: LoginState,
  accountExpiry: ?string,
  appVersion: string,
  consistentVersion: boolean,
  upToDateVersion: boolean,
  onQuit: () => void,
  onClose: () => void,
  onViewAccount: () => void,
  onViewSupport: () => void,
  onViewPreferences: () => void,
  onViewAdvancedSettings: () => void,
  onExternalLink: (type: string) => void,
  updateAccountExpiry: () => Promise<void>,
};*/


export default class Settings extends Component /*:: <Props>*/ {
  _windowStateObserver = new WindowStateObserver();

  componentDidMount() {
    this.props.updateAccountExpiry();

    this._windowStateObserver.onShow = () => {
      this.props.updateAccountExpiry();
    };
  }

  componentWillUnmount() {
    this._windowStateObserver.dispose();
  }

  render() {
    return <Layout>
        <Container>
          <View style={styles.settings}>
            <NavigationBar>
              <CloseBarItem action={this.props.onClose} />
            </NavigationBar>

            <View style={styles.settings__container}>
              <SettingsHeader>
                <HeaderTitle>Settings</HeaderTitle>
              </SettingsHeader>

              <CustomScrollbars style={styles.settings__scrollview} autoHide={true}>
                <View style={styles.settings__content}>
                  <View>
                    {this._renderTopButtons()}
                    {this._renderMiddleButtons()}
                    {this._renderBottomButtons()}
                  </View>
                  {this._renderQuitButton()}
                </View>
              </CustomScrollbars>
            </View>
          </View>
        </Container>
      </Layout>;
  }

  _renderTopButtons() {
    const isLoggedIn = this.props.loginState === 'ok';
    if (!isLoggedIn) {
      return null;
    }

    let isOutOfTime = false;
    let formattedExpiry = '';

    const expiryIso = this.props.accountExpiry;
    if (isLoggedIn && expiryIso) {
      const expiry = moment(expiryIso);
      isOutOfTime = expiry.isSameOrBefore(moment());
      formattedExpiry = (expiry.fromNow(true) + ' left').toUpperCase();
    }

    return <View>
        <View testName="settings__account">
          {isOutOfTime ? <Cell.CellButton onPress={this.props.onViewAccount} testName="settings__account_paid_until_button">
              <Cell.Label>Account</Cell.Label>
              <Cell.SubText testName="settings__account_paid_until_subtext" style={styles.settings__account_paid_until_label__error}>
                {'OUT OF TIME'}
              </Cell.SubText>
              <Cell.Img height={12} width={7} source="icon-chevron" />
            </Cell.CellButton> : <Cell.CellButton onPress={this.props.onViewAccount} testName="settings__account_paid_until_button">
              <Cell.Label>Account</Cell.Label>
              <Cell.SubText testName="settings__account_paid_until_subtext">
                {formattedExpiry}
              </Cell.SubText>
              <Cell.Img height={12} width={7} source="icon-chevron" />
            </Cell.CellButton>}
        </View>

        <Cell.CellButton onPress={this.props.onViewPreferences} testName="settings__preferences">
          <Cell.Label>Preferences</Cell.Label>
          <Cell.Img height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewAdvancedSettings} testName="settings__advanced">
          <Cell.Label>Advanced</Cell.Label>
          <Cell.Img height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
        <View style={styles.settings__cell_spacer} />
      </View>;
  }

  _renderMiddleButtons() {
    let icon;
    let footer;
    if (!this.props.consistentVersion || !this.props.upToDateVersion) {
      const message = !this.props.consistentVersion ? 'Inconsistent internal version information, please restart the app.' : 'This is not the latest version, download the update to remain safe.';

      icon = <Img source="icon-alert" tintColor={colors.red} style={styles.settings__version_warning} />;
      footer = <View style={styles.settings__cell_footer}>
          <Text style={styles.settings__cell_footer_label}>{message}</Text>
        </View>;
    } else {
      footer = <View style={styles.settings__cell_spacer} />;
    }

    return <View>
        <Cell.CellButton onPress={this.props.onExternalLink.bind(this, 'download')} testName="settings__version">
          {icon}
          <Cell.Label>App version</Cell.Label>
          <Cell.SubText>{this.props.appVersion}</Cell.SubText>
          <Cell.Img height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>
        {footer}
      </View>;
  }

  _renderBottomButtons() {
    return <View>
        <Cell.CellButton onPress={this.props.onExternalLink.bind(this, 'faq')} testName="settings__external_link">
          <Cell.Label>FAQs</Cell.Label>
          <Cell.Img height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onExternalLink.bind(this, 'guides')} testName="settings__external_link">
          <Cell.Label>Guides</Cell.Label>
          <Cell.Img height={16} width={16} source="icon-extLink" />
        </Cell.CellButton>

        <Cell.CellButton onPress={this.props.onViewSupport} testName="settings__view_support">
          <Cell.Label>Report a problem</Cell.Label>
          <Cell.Img height={12} width={7} source="icon-chevron" />
        </Cell.CellButton>
      </View>;
  }

  _renderQuitButton() {
    return <View style={styles.settings__footer}>
        <AppButton.RedButton onPress={this.props.onQuit} testName="settings__quit">
          <AppButton.Label>Quit app</AppButton.Label>
        </AppButton.RedButton>
      </View>;
  }
}