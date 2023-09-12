#pragma once

#include <cmath>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

template <typename T> struct Array : std::vector<T> {
    void push(T t) { this->push_back(t); }
    T pop() {
        const auto t = this->back();
        this->pop_back();
        return t;
    }
    T &last() { return this->back(); }
    T &first() { return this->front(); }
    T &operator[](int i) { return this->at(i); }
    int length() { return this->size(); }
    bool empty() { return this->size() == 0; }
    void clear() { this->clear(); }
    void resize(int n) { this->resize(n); }
    void reserve(int n) { this->reserve(n); }
    void remove(int i) { this->erase(this->begin() + i); }
    void insert(int i, T t) { this->insert(this->begin() + i, t); }
    void append(Array<T> a) {
        for (auto t : a) {
            {
                this->push(t);
            }
        }
    }
    void append(T t) { this->push(t); }
    Array<T> slice(int start, int end) {
        Array<T> a;
        for (int i = start; i < end; i++) {
            {
                a.push(this->at(i));
            }
        }
        return a;
    }
    Array<T> slice(int start) { return this->slice(start, this->size()); }

    Array<T>(std::initializer_list<T> l) : std::vector<T>(l) {}
    Array<T>() : std::vector<T>() {}
    Array<T>(int n) : std::vector<T>(n) {}
    Array<T>(int n, T t) : std::vector<T>(n, t) {}
    Array<T>(const Array<T> &a) : std::vector<T>(a) {}
    Array<T>(Array<T> &&a) : std::vector<T>(a) {}
    Array<T>(std::vector<T> v) : std::vector<T>(v) {}
    Array<T>(std::vector<T> &&v) : std::vector<T>(v) {}
    Array<T>(std::vector<T>::iterator begin, std::vector<T>::iterator end)
        : std::vector<T>(begin, end) {}
    Array<T>(std::vector<T>::iterator begin, int n)
        : std::vector<T>(begin, begin + n) {}
};

template <typename... Args> void println(std::string fmt, Args... args) {
    std::stringstream ss;
    int i = 0;
    while (i < fmt.length()) {
        if (fmt[i] == '{') {
            if (fmt[i + 1] == '}') {
                ((ss << std::forward<Args>(args)), ...);
                i += 2;
            } else if (fmt[i + 1] == '{') {
                ss << '{';
                i += 2;
            } else {
                throw std::runtime_error("invalid format string");
            }
        } else if (fmt[i] == '}') {
            if (fmt[i + 1] == '}') {
                ss << '}';
                i += 2;
            } else {
                throw std::runtime_error("invalid format string");
            }
        } else {
            ss << fmt[i];
            i++;
        }
    }
    std::cout << ss.str() << std::endl;
}
