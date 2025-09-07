import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import '../models/email_credentials.dart';
import '../models/email_account.dart';

class CredentialStorageService {
  static const FlutterSecureStorage _storage = FlutterSecureStorage(
    aOptions: AndroidOptions(
      encryptedSharedPreferences: true,
      sharedPreferencesName: 'meeru_secure_prefs',
    ),
    iOptions: IOSOptions(
      groupId: 'group.com.example.meeru',
      accountName: 'meeru_keychain',
    ),
    lOptions: LinuxOptions(),
    wOptions: WindowsOptions(useBackwardCompatibility: true),
    mOptions: MacOsOptions(
      groupId: 'group.com.example.meeru',
      accountName: 'meeru_keychain',
    ),
    webOptions: WebOptions(
      dbName: 'meeru_secure_storage',
      publicKey: 'meeru_public_key',
    ),
  );

  static const String _credentialsPrefix = 'credentials_';
  static const String _accountsKey = 'email_accounts';

  Future<void> storeCredentials(EmailCredentials credentials) async {
    try {
      final key = _credentialsPrefix + credentials.accountId;
      final jsonString = jsonEncode(credentials.toJson());
      await _storage.write(key: key, value: jsonString);
    } catch (e) {
      throw CredentialStorageException('Failed to store credentials: $e');
    }
  }

  Future<EmailCredentials?> getCredentials(String accountId) async {
    try {
      final key = _credentialsPrefix + accountId;
      final jsonString = await _storage.read(key: key);

      if (jsonString == null) {
        return null;
      }

      final json = jsonDecode(jsonString) as Map<String, dynamic>;
      return EmailCredentials.fromJson(json);
    } catch (e) {
      throw CredentialStorageException('Failed to retrieve credentials: $e');
    }
  }

  Future<void> deleteCredentials(String accountId) async {
    try {
      final key = _credentialsPrefix + accountId;
      await _storage.delete(key: key);
    } catch (e) {
      throw CredentialStorageException('Failed to delete credentials: $e');
    }
  }

  Future<void> updateCredentials(EmailCredentials credentials) async {
    await storeCredentials(credentials);
  }

  Future<void> storeAccounts(List<EmailAccount> accounts) async {
    try {
      final accountsJson = accounts.map((account) => account.toJson()).toList();
      final jsonString = jsonEncode(accountsJson);
      await _storage.write(key: _accountsKey, value: jsonString);
    } catch (e) {
      throw CredentialStorageException('Failed to store accounts: $e');
    }
  }

  Future<List<EmailAccount>> getAccounts() async {
    try {
      final jsonString = await _storage.read(key: _accountsKey);

      if (jsonString == null) {
        return [];
      }

      final jsonList = jsonDecode(jsonString) as List<dynamic>;
      return jsonList
          .cast<Map<String, dynamic>>()
          .map((json) => EmailAccount.fromJson(json))
          .toList();
    } catch (e) {
      throw CredentialStorageException('Failed to retrieve accounts: $e');
    }
  }

  Future<void> deleteAccount(String accountId) async {
    try {
      // Delete credentials for this account
      await deleteCredentials(accountId);

      // Remove account from the accounts list
      final accounts = await getAccounts();
      final updatedAccounts = accounts
          .where((account) => account.id != accountId)
          .toList();
      await storeAccounts(updatedAccounts);
    } catch (e) {
      throw CredentialStorageException('Failed to delete account: $e');
    }
  }

  Future<void> clearAll() async {
    try {
      await _storage.deleteAll();
    } catch (e) {
      throw CredentialStorageException('Failed to clear all data: $e');
    }
  }

  Future<bool> hasCredentials(String accountId) async {
    try {
      final key = _credentialsPrefix + accountId;
      final value = await _storage.read(key: key);
      return value != null;
    } catch (e) {
      return false;
    }
  }

  Future<Map<String, String>> getAllKeys() async {
    try {
      return await _storage.readAll();
    } catch (e) {
      throw CredentialStorageException('Failed to get all keys: $e');
    }
  }

  Future<bool> isStorageAvailable() async {
    try {
      await _storage.containsKey(key: 'test_key');
      return true;
    } catch (e) {
      return false;
    }
  }
}

class CredentialStorageException implements Exception {
  final String message;

  const CredentialStorageException(this.message);

  @override
  String toString() => 'CredentialStorageException: $message';
}
