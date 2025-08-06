class EmailCredentials {
  final String accountId;
  final String email;
  final String? password;
  final String? accessToken;
  final String? refreshToken;
  final DateTime? tokenExpiresAt;
  final Map<String, dynamic>? oauthTokens;

  const EmailCredentials({
    required this.accountId,
    required this.email,
    this.password,
    this.accessToken,
    this.refreshToken,
    this.tokenExpiresAt,
    this.oauthTokens,
  });

  EmailCredentials copyWith({
    String? accountId,
    String? email,
    String? password,
    String? accessToken,
    String? refreshToken,
    DateTime? tokenExpiresAt,
    Map<String, dynamic>? oauthTokens,
  }) {
    return EmailCredentials(
      accountId: accountId ?? this.accountId,
      email: email ?? this.email,
      password: password ?? this.password,
      accessToken: accessToken ?? this.accessToken,
      refreshToken: refreshToken ?? this.refreshToken,
      tokenExpiresAt: tokenExpiresAt ?? this.tokenExpiresAt,
      oauthTokens: oauthTokens ?? this.oauthTokens,
    );
  }

  bool get isPasswordAuth => password != null;
  bool get isOAuthAuth => accessToken != null;
  bool get isTokenExpired => tokenExpiresAt != null && 
      DateTime.now().isAfter(tokenExpiresAt!);

  Map<String, dynamic> toJson() {
    return {
      'accountId': accountId,
      'email': email,
      'password': password,
      'accessToken': accessToken,
      'refreshToken': refreshToken,
      'tokenExpiresAt': tokenExpiresAt?.toIso8601String(),
      'oauthTokens': oauthTokens,
    };
  }

  factory EmailCredentials.fromJson(Map<String, dynamic> json) {
    return EmailCredentials(
      accountId: json['accountId'],
      email: json['email'],
      password: json['password'],
      accessToken: json['accessToken'],
      refreshToken: json['refreshToken'],
      tokenExpiresAt: json['tokenExpiresAt'] != null 
          ? DateTime.parse(json['tokenExpiresAt']) 
          : null,
      oauthTokens: json['oauthTokens']?.cast<String, dynamic>(),
    );
  }
}