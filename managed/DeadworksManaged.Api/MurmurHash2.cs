using System.Runtime.CompilerServices;
using System.Text;

namespace DeadworksManaged.Api;

public static class MurmurHash2
{
	private const uint M = 0x5bd1e995u;
	private const int  R = 24;

	public static uint Hash(ReadOnlySpan<byte> data, uint seed)
	{
		uint len = (uint)data.Length;
		uint h = seed ^ len;

		int i = 0;
		while (data.Length - i >= 4)
		{
			uint k = (uint)(data[i] | (data[i + 1] << 8) | (data[i + 2] << 16) | (data[i + 3] << 24));

			k *= M;
			k ^= k >> R;
			k *= M;

			h *= M;
			h ^= k;

			i += 4;
		}

		int tail = data.Length - i;
		switch (tail)
		{
			case 3:
				h ^= (uint)data[i + 2] << 16;
				goto case 2;
			case 2:
				h ^= (uint)data[i + 1] << 8;
				goto case 1;
			case 1:
				h ^= data[i];
				h *= M;
				break;
		}

		h ^= h >> 13;
		h *= M;
		h ^= h >> 15;

		return h;
	}

	public static uint HashLowerCase(string text, uint seed)
	{
		int byteCount = Encoding.UTF8.GetByteCount(text);
		Span<byte> buf = byteCount <= 256 ? stackalloc byte[byteCount] : new byte[byteCount];
		Encoding.UTF8.GetBytes(text, buf);

		for (int i = 0; i < buf.Length; i++)
		{
			byte b = buf[i];
			if (b >= (byte)'A' && b <= (byte)'Z')
				buf[i] = (byte)(b + 32);
		}

		return Hash(buf, seed);
	}

	[MethodImpl(MethodImplOptions.AggressiveInlining)]
	public static uint HashStringCaseless(string text) => HashLowerCase(text, 0x3501A674u);
}
