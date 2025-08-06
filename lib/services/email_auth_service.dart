import 'dart:io';
import 'package:dio/dio.dart';
import '../models/email_account.dart';
import '../models/email_credentials.dart';
import '../models/provider_config.dart';
import 'credential_storage_service.dart';

class EmailAuthService {
  final Dio _dio = Dio();
  final CredentialStorageService _credentialStorage = CredentialStorageService();

  Future<AuthResult> authenticateWithPassword({
    required String email,
    required String password,
    required EmailProvider provider,
    ServerConfig? customImapConfig,
    ServerConfig? customSmtpConfig,
  }) async {
    try {
      final providerConfig = ProviderConfig.getConfig(provider);
      
      if (providerConfig == null && provider != EmailProvider.custom) {
        throw AuthException('Unsupported email provider');
      }

      final imapConfig = customImapConfig ?? providerConfig!.imapConfig;
      final smtpConfig = customSmtpConfig ?? providerConfig!.smtpConfig;

      // Test IMAP connection
      final imapResult = await _testImapConnection(
        email: email,
        password: password,
        config: imapConfig,
      );

      if (!imapResult.success) {
        throw AuthException('IMAP authentication failed: ${imapResult.error}');
      }

      // Test SMTP connection
      final smtpResult = await _testSmtpConnection(
        email: email,
        password: password,
        config: smtpConfig,
      );

      if (!smtpResult.success) {
        throw AuthException('SMTP authentication failed: ${smtpResult.error}');
      }

      // Create account
      final account = EmailAccount(
        id: _generateAccountId(email),
        name: _extractNameFromEmail(email),
        email: email,
        displayName: _extractNameFromEmail(email),
        provider: provider,
        imapConfig: imapConfig,
        smtpConfig: smtpConfig,
        createdAt: DateTime.now(),
      );

      // Create credentials
      final credentials = EmailCredentials(
        accountId: account.id,
        email: email,
        password: password,
      );

      // Store credentials and account
      await _credentialStorage.storeCredentials(credentials);
      
      final existingAccounts = await _credentialStorage.getAccounts();
      existingAccounts.add(account);
      await _credentialStorage.storeAccounts(existingAccounts);

      return AuthResult.success(account, credentials);
    } catch (e) {
      if (e is AuthException) {
        return AuthResult.failure(e.message);
      }
      return AuthResult.failure('Authentication failed: $e');
    }
  }

  Future<AuthResult> authenticateWithOAuth({
    required String email,
    required EmailProvider provider,
    required String authorizationCode,
  }) async {
    try {
      final providerConfig = ProviderConfig.getConfig(provider);
      
      if (providerConfig?.oauthConfig == null) {
        throw AuthException('OAuth not supported for this provider');
      }

      final oauthConfig = providerConfig!.oauthConfig!;

      // Exchange authorization code for tokens
      final tokenResponse = await _exchangeAuthorizationCode(
        authorizationCode: authorizationCode,
        oauthConfig: oauthConfig,
      );

      // Create account
      final account = EmailAccount(
        id: _generateAccountId(email),
        name: _extractNameFromEmail(email),
        email: email,
        displayName: _extractNameFromEmail(email),
        provider: provider,
        imapConfig: providerConfig.imapConfig.copyWith(
          authMethod: AuthMethod.oauth2,
        ),
        smtpConfig: providerConfig.smtpConfig.copyWith(
          authMethod: AuthMethod.oauth2,
        ),
        createdAt: DateTime.now(),
      );

      // Create credentials
      final credentials = EmailCredentials(
        accountId: account.id,
        email: email,
        accessToken: tokenResponse['access_token'],
        refreshToken: tokenResponse['refresh_token'],
        tokenExpiresAt: DateTime.now().add(
          Duration(seconds: tokenResponse['expires_in'] ?? 3600),
        ),
        oauthTokens: tokenResponse,
      );

      // Store credentials and account
      await _credentialStorage.storeCredentials(credentials);
      
      final existingAccounts = await _credentialStorage.getAccounts();
      existingAccounts.add(account);
      await _credentialStorage.storeAccounts(existingAccounts);

      return AuthResult.success(account, credentials);
    } catch (e) {
      if (e is AuthException) {
        return AuthResult.failure(e.message);
      }
      return AuthResult.failure('OAuth authentication failed: $e');
    }
  }

  Future<bool> refreshToken(String accountId) async {
    try {
      final credentials = await _credentialStorage.getCredentials(accountId);
      
      if (credentials == null || credentials.refreshToken == null) {
        return false;
      }

      final accounts = await _credentialStorage.getAccounts();
      final account = accounts.where((a) => a.id == accountId).firstOrNull;
      
      if (account == null) {
        return false;
      }

      final providerConfig = ProviderConfig.getConfig(account.provider);
      if (providerConfig?.oauthConfig == null) {
        return false;
      }

      final newTokens = await _refreshAccessToken(
        refreshToken: credentials.refreshToken!,
        oauthConfig: providerConfig!.oauthConfig!,
      );

      final updatedCredentials = credentials.copyWith(
        accessToken: newTokens['access_token'],
        refreshToken: newTokens['refresh_token'] ?? credentials.refreshToken,
        tokenExpiresAt: DateTime.now().add(
          Duration(seconds: newTokens['expires_in'] ?? 3600),
        ),
        oauthTokens: {...?credentials.oauthTokens, ...newTokens},
      );

      await _credentialStorage.storeCredentials(updatedCredentials);
      return true;
    } catch (e) {
      return false;
    }
  }

  Future<ConnectionTestResult> _testImapConnection({
    required String email,
    required String password,
    required ServerConfig config,
  }) async {
    try {
      // Basic socket connection test
      final socket = await Socket.connect(
        config.host, 
        config.port,
        timeout: const Duration(seconds: 10),
      );
      
      socket.destroy();
      
      // For now, we'll consider a successful socket connection as valid
      // In a real implementation, you would use an IMAP library here
      return ConnectionTestResult.success();
    } catch (e) {
      return ConnectionTestResult.failure('Failed to connect to IMAP server: $e');
    }
  }

  Future<ConnectionTestResult> _testSmtpConnection({
    required String email,
    required String password,
    required ServerConfig config,
  }) async {
    try {
      // Basic socket connection test
      final socket = await Socket.connect(
        config.host, 
        config.port,
        timeout: const Duration(seconds: 10),
      );
      
      socket.destroy();
      
      // For now, we'll consider a successful socket connection as valid
      // In a real implementation, you would use an SMTP library here
      return ConnectionTestResult.success();
    } catch (e) {
      return ConnectionTestResult.failure('Failed to connect to SMTP server: $e');
    }
  }

  Future<Map<String, dynamic>> _exchangeAuthorizationCode({
    required String authorizationCode,
    required OAuthConfig oauthConfig,
  }) async {
    final response = await _dio.post(
      oauthConfig.tokenUrl,
      data: {
        'grant_type': 'authorization_code',
        'code': authorizationCode,
        'client_id': oauthConfig.clientId,
        'client_secret': oauthConfig.clientSecret,
        'redirect_uri': oauthConfig.redirectUri,
      },
      options: Options(
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      ),
    );

    return response.data;
  }

  Future<Map<String, dynamic>> _refreshAccessToken({
    required String refreshToken,
    required OAuthConfig oauthConfig,
  }) async {
    final response = await _dio.post(
      oauthConfig.tokenUrl,
      data: {
        'grant_type': 'refresh_token',
        'refresh_token': refreshToken,
        'client_id': oauthConfig.clientId,
        'client_secret': oauthConfig.clientSecret,
      },
      options: Options(
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
      ),
    );

    return response.data;
  }

  String _generateAccountId(String email) {
    return '${email}_${DateTime.now().millisecondsSinceEpoch}';
  }

  String _extractNameFromEmail(String email) {
    final parts = email.split('@');
    return parts.isNotEmpty ? parts.first : email;
  }
}

class AuthResult {
  final bool success;
  final EmailAccount? account;
  final EmailCredentials? credentials;
  final String? error;

  AuthResult._({
    required this.success,
    this.account,
    this.credentials,
    this.error,
  });

  factory AuthResult.success(EmailAccount account, EmailCredentials credentials) {
    return AuthResult._(
      success: true,
      account: account,
      credentials: credentials,
    );
  }

  factory AuthResult.failure(String error) {
    return AuthResult._(
      success: false,
      error: error,
    );
  }
}

class ConnectionTestResult {
  final bool success;
  final String? error;

  ConnectionTestResult._({required this.success, this.error});

  factory ConnectionTestResult.success() {
    return ConnectionTestResult._(success: true);
  }

  factory ConnectionTestResult.failure(String error) {
    return ConnectionTestResult._(success: false, error: error);
  }
}

class AuthException implements Exception {
  final String message;
  
  const AuthException(this.message);
  
  @override
  String toString() => 'AuthException: $message';
}