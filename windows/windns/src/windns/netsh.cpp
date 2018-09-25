#include "stdafx.h"
#include "netsh.h"
#include "libcommon/applicationrunner.h"
#include "libcommon/string.h"
#include "libcommon/filesystem.h"
#include <sstream>
#include <stdexcept>
#include <experimental/filesystem>

namespace
{

ErrorSinkInfo g_ErrorSink = { nullptr, nullptr };

std::wstring g_NetShPath;

void InitializePath()
{
	if (false == g_NetShPath.empty())
	{
		return;
	}

	const auto system32 = common::fs::GetKnownFolderPath(FOLDERID_System, 0, nullptr);

	g_NetShPath = std::experimental::filesystem::path(system32).append(L"netsh.exe");
}

const std::wstring &NetShPath()
{
	InitializePath();

	return g_NetShPath;
}

void InfoSink(const char *msg)
{
	auto infoMsg = std::string("INFO: ").append(msg);

	g_ErrorSink.sink(infoMsg.c_str(), nullptr, 0, g_ErrorSink.context);
}

std::vector<std::string> BlockToRows(const std::string &textBlock)
{
	//
	// TODO: Formalize and move to libcommon.
	// There is a recurring need to split a text block into lines, ignoring blank lines.
	//
	// Also, changing the encoding back and forth is terribly wasteful.
	// Should look into replacing all of this with Boost some day.
	//

	const auto wideTextBlock = common::string::ToWide(textBlock);
	const auto wideRows = common::string::Tokenize(wideTextBlock, L"\r\n");

	std::vector<std::string> result;

	result.reserve(wideRows.size());

	std::transform(wideRows.begin(), wideRows.end(), std::back_inserter(result), [](const std::wstring &str)
	{
		return common::string::ToAnsi(str);
	});

	return result;
}

__declspec(noreturn) void ThrowWithDetails(std::string &&error, common::ApplicationRunner &netsh)
{
	std::vector<std::string> details { "Failed to capture output from 'netsh'" };

	std::string output;

	static const size_t MAX_CHARS = 2048;
	static const size_t TIMEOUT_MILLISECONDS = 2000;

	if (netsh.read(output, MAX_CHARS, TIMEOUT_MILLISECONDS))
	{
		auto outputRows = BlockToRows(output);

		if (false == outputRows.empty())
		{
			details = std::move(outputRows);
		}
	}

	throw NetShError(std::move(error), std::move(details));
}

void ValidateShellOut(common::ApplicationRunner &netsh, uint32_t timeout)
{
	// Use default timeout of 4 seconds.
	const uint32_t actualTimeout = (0 == timeout ? 3000 : timeout);

	const auto startTime = GetTickCount64();

	DWORD returnCode;

	if (false == netsh.join(returnCode, actualTimeout))
	{
		ThrowWithDetails("'netsh' did not complete in a timely manner", netsh);
	}

	if (returnCode != 0)
	{
		std::stringstream ss;

		ss << "'netsh' failed the requested operation. Error: " << returnCode;

		ThrowWithDetails(ss.str(), netsh);
	}

	const auto elapsed = static_cast<uint32_t>(GetTickCount64() - startTime);

	if (elapsed > (actualTimeout / 2))
	{
		std::stringstream ss;

		ss << L"'netsh' completed successfully, albeit a little slowly. It consumed "
			<< elapsed << " ms of "
			<< actualTimeout << " ms max permitted execution time";

		InfoSink(ss.str().c_str());
	}
}

} // anonymous namespace

//static
void NetSh::RegisterErrorSink(const ErrorSinkInfo &errorSink)
{
	g_ErrorSink = errorSink;
}

//static
void NetSh::SetIpv4PrimaryDns(uint32_t interfaceIndex, std::wstring server, uint32_t timeout)
{
	//
	// netsh interface ipv4 set dnsservers name="Ethernet 2" source=static address=8.8.8.8 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv4 set dnsservers name="
		<< interfaceIndex
		<< L" source=static address="
		<< server
		<< L" validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(NetShPath(), ss.str());

	ValidateShellOut(*netsh, timeout);
}

//static
void NetSh::SetIpv4SecondaryDns(uint32_t interfaceIndex, std::wstring server, uint32_t timeout)
{
	//
	// netsh interface ipv4 add dnsservers name="Ethernet 2" address=8.8.4.4 index=2 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv4 add dnsservers name="
		<< interfaceIndex
		<< L" address="
		<< server
		<< L" index=2 validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(NetShPath(), ss.str());

	ValidateShellOut(*netsh, timeout);
}

//static
void NetSh::SetIpv4Dhcp(uint32_t interfaceIndex, uint32_t timeout)
{
	//
	// netsh interface ipv4 set dnsservers name="Ethernet 2" source=dhcp
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv4 set dnsservers name="
		<< interfaceIndex
		<< L" source=dhcp";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(NetShPath(), ss.str());

	ValidateShellOut(*netsh, timeout);
}

//static
void NetSh::SetIpv6PrimaryDns(uint32_t interfaceIndex, std::wstring server, uint32_t timeout)
{
	//
	// netsh interface ipv6 set dnsservers name="Ethernet 2" source=static address=2001:4860:4860::8888 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv6 set dnsservers name="
		<< interfaceIndex
		<< L" source=static address="
		<< server
		<< L" validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(NetShPath(), ss.str());

	ValidateShellOut(*netsh, timeout);
}

//static
void NetSh::SetIpv6SecondaryDns(uint32_t interfaceIndex, std::wstring server, uint32_t timeout)
{
	//
	// netsh interface ipv6 add dnsservers name="Ethernet 2" address=2001:4860:4860::8844 index=2 validate=no
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv6 add dnsservers name="
		<< interfaceIndex
		<< L"address ="
		<< server
		<< L" index=2 validate=no";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(NetShPath(), ss.str());

	ValidateShellOut(*netsh, timeout);
}

//static
void NetSh::SetIpv6Dhcp(uint32_t interfaceIndex, uint32_t timeout)
{
	//
	// netsh interface ipv6 set dnsservers name="Ethernet 2" source=dhcp
	//
	// Note: we're specifying the interface by index instead.
	//

	std::wstringstream ss;

	ss << L"interface ipv6 set dnsservers name="
		<< interfaceIndex
		<< L" source=dhcp";

	auto netsh = common::ApplicationRunner::StartWithoutConsole(NetShPath(), ss.str());

	ValidateShellOut(*netsh, timeout);
}
