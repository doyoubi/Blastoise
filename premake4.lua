solution "Blastoise"
    configurations { "Debug", "Release" }
    location "build"
    includedirs "src"

    project "test"
        kind "ConsoleApp"
        language "C++"
        files { "test/**.h", "test/**.cpp", "src/**.cpp" }
        targetdir "build/test"
        targetname "test_program"
        flags { "Symbols" }
        buildoptions { "-g", "-std=c++0x" }
        linkoptions { "-lpthread", "-lgtest", "-lgtest_main" }
    
    project "BltServer"
        kind "ConsoleApp"
        language "C++"
        files { "**.h", "**.cpp" }
        
        configuration "Debug"
            defines { "DEBUG" }
            flags { "Symbols" }
        
        configuration "Release"
            defines { "NDEBUG" }
            flags { "Optimize" }

    project "BltClient"
        kind "ConsoleApp"
        language "C++"

        files { "**.h", "**.cpp" }

        configuration "Debug"
            defines { "DEBUG" }
            flags { "Symbols" }

        configuration "Release"
            defines { "NDEBUG" }
            flags { "Optimize" }
