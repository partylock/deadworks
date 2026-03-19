namespace DeadworksManaged.Api;

public enum EGameState : uint {
	Invalid = 0x0,
	Init = 0x1,
	WaitingForPlayersToJoin = 0x2,
	HeroSelection = 0x3,
	MatchIntro = 0x4,
	WaitForMapToLoad = 0x5,
	PreGameWait = 0x6,
	GameInProgress = 0x7,
	PostGame = 0x8,
	PostGamePlayOfTheGame = 0x9,
	Abandoned = 0xa,
	End = 0xb,
}

public enum ECitadelMatchMode : uint {
	Invalid = 0x0,
	Unranked = 0x1,
	PrivateLobby = 0x2,
	CoopBot = 0x3,
	Ranked = 0x4,
	ServerTest = 0x5,
	Tutorial = 0x6,
	HeroLabs = 0x7,
	Calibration = 0x8,
}

public enum ECitadelGameMode : uint {
	Invalid = 0x0,
	Normal = 0x1,
	OneVOneTest = 0x2,
	Sandbox = 0x3,
	StreetBrawl = 0x4,
	ExploreNYC = 0x5,
	Internal = 0x6,
}
