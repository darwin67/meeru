import 'package:flutter/foundation.dart';
import '../models/email_account.dart';
import '../models/email_credentials.dart';
import '../services/email_auth_service.dart';
import '../services/credential_storage_service.dart';

class AuthProvider with ChangeNotifier {
  final EmailAuthService _authService = EmailAuthService();
  final CredentialStorageService _credentialStorage =
      CredentialStorageService();

  List<EmailAccount> _accounts = [];
  EmailAccount? _currentAccount;
  bool _isLoading = false;
  String? _error;

  List<EmailAccount> get accounts => _accounts;
  EmailAccount? get currentAccount => _currentAccount;
  bool get isLoading => _isLoading;
  String? get error => _error;
  bool get hasAccounts => _accounts.isNotEmpty;

  Future<void> loadAccounts() async {
    _setLoading(true);
    try {
      _accounts = await _credentialStorage.getAccounts();
      if (_accounts.isNotEmpty && _currentAccount == null) {
        _currentAccount = _accounts.first;
      }
      _clearError();
    } catch (e) {
      _setError('Failed to load accounts: $e');
    }
    _setLoading(false);
  }

  Future<bool> authenticateWithPassword({
    required String email,
    required String password,
    required EmailProvider provider,
    ServerConfig? customImapConfig,
    ServerConfig? customSmtpConfig,
  }) async {
    _setLoading(true);
    try {
      final result = await _authService.authenticateWithPassword(
        email: email,
        password: password,
        provider: provider,
        customImapConfig: customImapConfig,
        customSmtpConfig: customSmtpConfig,
      );

      if (result.success && result.account != null) {
        _accounts.add(result.account!);
        _currentAccount = result.account;
        _clearError();
        notifyListeners();
        return true;
      } else {
        _setError(result.error ?? 'Authentication failed');
        return false;
      }
    } catch (e) {
      _setError('Authentication failed: $e');
      return false;
    } finally {
      _setLoading(false);
    }
  }

  Future<bool> authenticateWithOAuth({
    required String email,
    required EmailProvider provider,
    required String authorizationCode,
  }) async {
    _setLoading(true);
    try {
      final result = await _authService.authenticateWithOAuth(
        email: email,
        provider: provider,
        authorizationCode: authorizationCode,
      );

      if (result.success && result.account != null) {
        _accounts.add(result.account!);
        _currentAccount = result.account;
        _clearError();
        notifyListeners();
        return true;
      } else {
        _setError(result.error ?? 'OAuth authentication failed');
        return false;
      }
    } catch (e) {
      _setError('OAuth authentication failed: $e');
      return false;
    } finally {
      _setLoading(false);
    }
  }

  Future<void> switchAccount(EmailAccount account) async {
    if (_accounts.contains(account)) {
      _currentAccount = account;
      notifyListeners();
    }
  }

  Future<void> removeAccount(String accountId) async {
    try {
      await _credentialStorage.deleteAccount(accountId);
      _accounts.removeWhere((account) => account.id == accountId);

      if (_currentAccount?.id == accountId) {
        _currentAccount = _accounts.isNotEmpty ? _accounts.first : null;
      }

      notifyListeners();
    } catch (e) {
      _setError('Failed to remove account: $e');
    }
  }

  Future<void> refreshTokens() async {
    for (final account in _accounts) {
      if (account.imapConfig.authMethod == AuthMethod.oauth2) {
        await _authService.refreshToken(account.id);
      }
    }
  }

  Future<EmailCredentials?> getCredentialsForAccount(String accountId) async {
    try {
      return await _credentialStorage.getCredentials(accountId);
    } catch (e) {
      _setError('Failed to get credentials: $e');
      return null;
    }
  }

  void clearError() {
    _clearError();
  }

  Future<void> clearAllData() async {
    try {
      await _credentialStorage.clearAll();
      _accounts = [];
      _currentAccount = null;
      _clearError();
    } catch (e) {
      _setError('Failed to clear all data: $e');
    }
  }

  void _setLoading(bool loading) {
    _isLoading = loading;
    notifyListeners();
  }

  void _setError(String error) {
    _error = error;
    notifyListeners();
  }

  void _clearError() {
    _error = null;
    notifyListeners();
  }
}
