import 'email_account.dart';

class ProviderConfig {
  final EmailProvider provider;
  final String name;
  final String displayName;
  final ServerConfig imapConfig;
  final ServerConfig smtpConfig;
  final List<AuthMethod> supportedAuthMethods;
  final OAuthConfig? oauthConfig;

  const ProviderConfig({
    required this.provider,
    required this.name,
    required this.displayName,
    required this.imapConfig,
    required this.smtpConfig,
    required this.supportedAuthMethods,
    this.oauthConfig,
  });

  static const List<ProviderConfig> predefinedProviders = [
    ProviderConfig(
      provider: EmailProvider.gmail,
      name: 'gmail',
      displayName: 'Gmail',
      imapConfig: ServerConfig(
        host: 'imap.gmail.com',
        port: 993,
        useSSL: true,
        useTLS: false,
        authMethod: AuthMethod.oauth2,
      ),
      smtpConfig: ServerConfig(
        host: 'smtp.gmail.com',
        port: 587,
        useSSL: false,
        useTLS: true,
        authMethod: AuthMethod.oauth2,
      ),
      supportedAuthMethods: [
        AuthMethod.oauth2,
        AuthMethod.appPassword,
      ],
      oauthConfig: OAuthConfig(
        clientId: '', // To be configured
        scopes: [
          'https://www.googleapis.com/auth/gmail.readonly',
          'https://www.googleapis.com/auth/gmail.send',
          'https://www.googleapis.com/auth/gmail.modify',
        ],
        authUrl: 'https://accounts.google.com/o/oauth2/auth',
        tokenUrl: 'https://oauth2.googleapis.com/token',
      ),
    ),
    ProviderConfig(
      provider: EmailProvider.outlook,
      name: 'outlook',
      displayName: 'Outlook/Hotmail',
      imapConfig: ServerConfig(
        host: 'outlook.office365.com',
        port: 993,
        useSSL: true,
        useTLS: false,
        authMethod: AuthMethod.oauth2,
      ),
      smtpConfig: ServerConfig(
        host: 'smtp-mail.outlook.com',
        port: 587,
        useSSL: false,
        useTLS: true,
        authMethod: AuthMethod.oauth2,
      ),
      supportedAuthMethods: [
        AuthMethod.oauth2,
        AuthMethod.appPassword,
      ],
      oauthConfig: OAuthConfig(
        clientId: '', // To be configured
        scopes: [
          'https://graph.microsoft.com/IMAP.AccessAsUser.All',
          'https://graph.microsoft.com/SMTP.Send',
        ],
        authUrl: 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
        tokenUrl: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
      ),
    ),
    ProviderConfig(
      provider: EmailProvider.yahoo,
      name: 'yahoo',
      displayName: 'Yahoo Mail',
      imapConfig: ServerConfig(
        host: 'imap.mail.yahoo.com',
        port: 993,
        useSSL: true,
        useTLS: false,
        authMethod: AuthMethod.appPassword,
      ),
      smtpConfig: ServerConfig(
        host: 'smtp.mail.yahoo.com',
        port: 587,
        useSSL: false,
        useTLS: true,
        authMethod: AuthMethod.appPassword,
      ),
      supportedAuthMethods: [
        AuthMethod.appPassword,
      ],
    ),
    ProviderConfig(
      provider: EmailProvider.icloud,
      name: 'icloud',
      displayName: 'iCloud',
      imapConfig: ServerConfig(
        host: 'imap.mail.me.com',
        port: 993,
        useSSL: true,
        useTLS: false,
        authMethod: AuthMethod.appPassword,
      ),
      smtpConfig: ServerConfig(
        host: 'smtp.mail.me.com',
        port: 587,
        useSSL: false,
        useTLS: true,
        authMethod: AuthMethod.appPassword,
      ),
      supportedAuthMethods: [
        AuthMethod.appPassword,
      ],
    ),
  ];

  static ProviderConfig? getConfig(EmailProvider provider) {
    try {
      return predefinedProviders.firstWhere((config) => config.provider == provider);
    } catch (e) {
      return null;
    }
  }
}

class OAuthConfig {
  final String clientId;
  final String? clientSecret;
  final List<String> scopes;
  final String authUrl;
  final String tokenUrl;
  final String redirectUri;

  const OAuthConfig({
    required this.clientId,
    this.clientSecret,
    required this.scopes,
    required this.authUrl,
    required this.tokenUrl,
    this.redirectUri = 'com.example.meeru://oauth',
  });
}